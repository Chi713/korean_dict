from koreanDict import koreanDictApi
import asyncio
import cProfile
import pstats

#query = "나무"
#test_sentence = '제 친구 정우가 공항에서 저와 줄리아를 기다리고 있었어요.'
#test_sentence = '어절 내에 포함된 형태소의 리스트를 생성한다'
#test_sentence = '사람의 말이나 행동, 성격에서 잘못된 점이나 부족한 점을 지적하여 말하다.'

kr = koreanDictApi

if __name__ == '__main__':
    #with cProfile.Profile() as pr:
        #query = "나무"
        #test_sentence = '제 친구 정우가 공항에서 저와 줄리아를 기다리고 있었어요.'
        #test_sentence = '어절 내에 포함된 형태소의 리스트를 생성한다'
        #test_sentence = '사람의 말이나 행동, 성격에서 잘못된 점이나 부족한 점을 지적하여 말하다.'

        file = "resources/bite_sisters.tsv"

        asyncio.run(kr.main(file))
    #stats = pstats.Stats(pr)
    #stats.sort_stats(pstats.SortKey.TIME)
    #stats.print_stats()
