import {
  Injectable,
  NestInterceptor,
  ExecutionContext,
  CallHandler,
  HttpStatus,
  ConflictException,
} from '@nestjs/common';
import { Observable, from, throwError } from 'rxjs';
import { catchError, mergeMap, tap } from 'rxjs/operators';
import { Request, Response } from 'express';
import { Reflector } from '@nestjs/core';
import {
  IDEMPOTENCY_KEY_OPTIONS,
  IdempotencyOptions,
} from './idempotency.constants';
import { IdempotencyService } from './idempotency.service';
import type { CapturedHttpResponse } from './idempotency.service';

@Injectable()
export class IdempotencyInterceptor implements NestInterceptor {
  constructor(
    private readonly reflector: Reflector,
    private readonly idempotencyService: IdempotencyService,
  ) {}

  async intercept(
    context: ExecutionContext,
    next: CallHandler,
  ): Promise<Observable<any>> {
    const request = context.switchToHttp().getRequest<Request>();
    const response = context.switchToHttp().getResponse<Response>();

    const options =
      this.reflector.get<IdempotencyOptions>(
        IDEMPOTENCY_KEY_OPTIONS,
        context.getHandler(),
      ) || {};

    if (!this.isIdempotentMethod(request.method)) {
      return next.handle();
    }

    // Check if this request has been processed before
    const existingRecord = await this.idempotencyService.checkIdempotency(
      request,
      options,
    );

    if (existingRecord) {
      if (existingRecord.status === 'in_flight') {
        throw new ConflictException({
          error: 'IDEMPOTENCY_IN_PROGRESS',
          message: 'Request is currently being processed',
        });
      }

      // Validate request integrity
      const isValid = this.idempotencyService.validateRequestIntegrity(
        request,
        existingRecord,
        options,
      );

      if (!isValid) {
        throw new ConflictException({
          error: 'IDEMPOTENCY_MISMATCH',
          message:
            'Request content differs from original request with same idempotency key',
        });
      }

      // Return cached response
      if (existingRecord.response) {
        Object.entries(existingRecord.response.headers).forEach(
          ([key, value]) => {
            if (!key.toLowerCase().startsWith('x-')) {
              response.set(key, value);
            }
          },
        );

        response.set('X-Idempotent-Replayed', 'true');
        response.status(existingRecord.response.statusCode);

        return new Observable((subscriber) => {
          subscriber.next(existingRecord.response?.body ?? null);
          subscriber.complete();
        });
      }
    }

    // Mark as in-progress
    await this.idempotencyService.markInFlight(request, options);

    return next.handle().pipe(
      tap(async (data) => {
        const statusCode = response.statusCode || HttpStatus.OK;
        const captured: CapturedHttpResponse = {
          statusCode,
          getHeaders: () => response.getHeaders(),
          body: data,
        };
        await this.idempotencyService.storeResponse(
          request,
          captured,
          options,
        );
        response.set('X-Idempotent', 'true');
      }),
      catchError(async (error) => {
        const statusCode = error.status || HttpStatus.INTERNAL_SERVER_ERROR;
        const capturedErr: CapturedHttpResponse = {
          statusCode,
          getHeaders: () => response.getHeaders(),
          body: {
            error: error.response?.error || 'INTERNAL_ERROR',
            message: error.message,
            timestamp: new Date().toISOString(),
          },
        };
        await this.idempotencyService.storeResponse(
          request,
          capturedErr,
          options,
        );
        response.set('X-Idempotent', 'true');
        return throwError(() => error);
      }),
    );
  }

  private isIdempotentMethod(method: string): boolean {
    const idempotentMethods = [
      'POST',
      'PUT',
      'DELETE',
      'PATCH',
    ];
    return idempotentMethods.includes(method.toUpperCase());
  }
}
