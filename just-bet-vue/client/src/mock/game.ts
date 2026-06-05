import { mockToken } from '@/mock'

export const game = {
  LOST: {
    id: '123isadfin12nxaowdo1',
    belognsTo: 'test',
    state: 'done',
    size: 25,
    bomb: {
      count: 5,
      position: [12, 23, 1, 4, 5]
    },
    stake: 3,
    clicks: [
      { position: 8, earned: 0.5 },
      { position: 9, earned: 0.75 },
      { position: 1, earned: 0.5 }
    ],
    pool: 100,
    nextClick: 1,
    result: 'lost'
  }
}
