from khaiii import KhaiiiApi
#import re

api = KhaiiiApi()
exceptionTags = ['NNP','NP']

def parse(text):
    morphemes = []
    tags = []

    for entry in [x for x in api.analyze(text) if x.morphs[0].tag not in exceptionTags]:
        word = entry.morphs[0].lex
        tag = entry.morphs[0].tag
        if 'V' in tag[0]:
            word = word + '다'
            #print(word)
        morphemes.append(word)
        tags.append(tag)
        #print(entry.morphs[0].lex, entry.morphs[0].tag)

    sentence = {'morpheme': morphemes,
        'tag': tags}

    #print(morphemes,"\n",tags)
    #print(sentence)
    return sentence
