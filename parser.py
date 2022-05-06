from khaiii import KhaiiiApi
#import re

api = KhaiiiApi()
morphemes = []

for word in api.analyze('제 친구 정우가 공항에서 저와 줄리아를 기다리고 있었어요.'):
    
    print(word.morphs[0].lex,word.morphs[0].tag)
#print(morphemes[0],morphemes[1])
