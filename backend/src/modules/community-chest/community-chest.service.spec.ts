import { Test, TestingModule } from '@nestjs/testing';
import { getRepositoryToken } from '@nestjs/typeorm';
import { CommunityChestService } from './community-chest.service';
import { CommunityChest } from './entities/community-chest.entity';
import {
  GetCommunityChestListDto,
  CommunityChestSortBy,
  SortOrder,
} from './dto/get-community-chest-list.dto';
import { RANDOM_PROVIDER, SeededRandomProvider } from '../../common/random-provider';

const mockCommunityChest = {
  id: 1,
  instruction: 'Advance to Go',
  type: 'advance_to_go',
  amount: 0,
  position: 0,
  extra: null,
};

const mockCommunityChest2 = {
  id: 2,
  instruction: 'Go to Jail',
  type: 'go_to_jail',
  amount: 0,
  position: 0,
  extra: null,
};

describe('CommunityChestService', () => {
  let service: CommunityChestService;
  let mockCreateQueryBuilder: jest.Mock;
  let mockQueryBuilder: {
    andWhere: jest.Mock;
    orderBy: jest.Mock;
    limit: jest.Mock;
    getOne: jest.Mock;
    getMany: jest.Mock;
  };
  let mockCount: jest.Mock;
  let mockFind: jest.Mock;

  beforeEach(async () => {
    mockQueryBuilder = {
      andWhere: jest.fn().mockReturnThis(),
      orderBy: jest.fn().mockReturnThis(),
      limit: jest.fn().mockReturnThis(),
      getOne: jest.fn().mockResolvedValue(mockCommunityChest),
      getMany: jest.fn().mockResolvedValue([mockCommunityChest]),
    };

    mockCreateQueryBuilder = jest.fn().mockReturnValue(mockQueryBuilder);
    mockCount = jest.fn().mockResolvedValue(2);
    mockFind = jest.fn().mockResolvedValue([mockCommunityChest]);

    const module: TestingModule = await Test.createTestingModule({
      providers: [
        CommunityChestService,
        {
          provide: getRepositoryToken(CommunityChest),
          useValue: {
            createQueryBuilder: mockCreateQueryBuilder,
            count: mockCount,
            find: mockFind,
          },
        },
        { provide: RANDOM_PROVIDER, useValue: new SeededRandomProvider(0) },
      ],
    }).compile();

    service = module.get<CommunityChestService>(CommunityChestService);
  });

  it('should be defined', () => {
    expect(service).toBeDefined();
  });

  describe('drawCard', () => {
    it('should return a random community chest card', async () => {
      const result = await service.drawCard();
      expect(result).toEqual(mockCommunityChest);
      expect(mockCount).toHaveBeenCalled();
      expect(mockFind).toHaveBeenCalledWith({
        order: { id: 'ASC' },
        skip: 0,
        take: 1,
      });
    });

    it('should return null when no cards exist', async () => {
      mockCount.mockResolvedValueOnce(0);
      const result = await service.drawCard();
      expect(result).toBeNull();
      expect(mockFind).not.toHaveBeenCalled();
    });

    it('drawCard is deterministic for a fixed seed', async () => {
      // Build two services with the same seed and identical repo state
      const buildSvc = async (seed: number) => {
        const mod = await Test.createTestingModule({
          providers: [
            CommunityChestService,
            {
              provide: getRepositoryToken(CommunityChest),
              useValue: { count: jest.fn().mockResolvedValue(5), find: jest.fn().mockImplementation(({ skip }: any) => Promise.resolve([{ id: skip + 1 }])) },
            },
            { provide: RANDOM_PROVIDER, useValue: new SeededRandomProvider(seed) },
          ],
        }).compile();
        return mod.get<CommunityChestService>(CommunityChestService);
      };

      const svc1 = await buildSvc(99);
      const svc2 = await buildSvc(99);
      const r1 = await svc1.drawCard();
      const r2 = await svc2.drawCard();
      expect(r1?.id).toBe(r2?.id);
    });
  });

  describe('findAll', () => {
    it('should return all community chest cards with default sorting', async () => {
      const query: GetCommunityChestListDto = {};
      mockQueryBuilder.getMany.mockResolvedValue([
        mockCommunityChest,
        mockCommunityChest2,
      ]);

      const result = await service.findAll(query);

      expect(result).toEqual([mockCommunityChest, mockCommunityChest2]);
      expect(mockCreateQueryBuilder).toHaveBeenCalledWith('community_chest');
      expect(mockQueryBuilder.orderBy).toHaveBeenCalledWith(
        'community_chest.id',
        'ASC',
      );
    });

    it('should sort by specified field in descending order', async () => {
      const query: GetCommunityChestListDto = {
        sortBy: CommunityChestSortBy.CREATED_AT,
        sortOrder: SortOrder.DESC,
      };

      await service.findAll(query);

      expect(mockQueryBuilder.orderBy).toHaveBeenCalledWith(
        'community_chest.createdAt',
        'DESC',
      );
    });

    it('should filter by type when provided', async () => {
      const query: GetCommunityChestListDto = {
        type: 'advance_to_go',
      };

      await service.findAll(query);

      expect(mockQueryBuilder.andWhere).toHaveBeenCalledWith(
        'community_chest.type = :type',
        { type: 'advance_to_go' },
      );
      expect(mockQueryBuilder.getMany).toHaveBeenCalled();
    });

    it('should handle invalid sortBy gracefully by defaulting to id', async () => {
      const query: GetCommunityChestListDto = {
        sortBy: 'invalidSort' as CommunityChestSortBy,
      };

      await service.findAll(query);

      expect(mockQueryBuilder.orderBy).toHaveBeenCalledWith(
        'community_chest.id',
        'ASC',
      );
    });

    it('should sort by type field', async () => {
      const query: GetCommunityChestListDto = {
        sortBy: CommunityChestSortBy.TYPE,
        sortOrder: SortOrder.ASC,
      };

      await service.findAll(query);

      expect(mockQueryBuilder.orderBy).toHaveBeenCalledWith(
        'community_chest.type',
        'ASC',
      );
    });

    it('should combine type filter with custom sorting', async () => {
      const query: GetCommunityChestListDto = {
        type: 'go_to_jail',
        sortBy: CommunityChestSortBy.AMOUNT,
        sortOrder: SortOrder.DESC,
      };

      await service.findAll(query);

      expect(mockQueryBuilder.andWhere).toHaveBeenCalledWith(
        'community_chest.type = :type',
        { type: 'go_to_jail' },
      );
      expect(mockQueryBuilder.orderBy).toHaveBeenCalledWith(
        'community_chest.amount',
        'DESC',
      );
    });
  });
});
