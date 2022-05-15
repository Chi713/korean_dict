from khaiii import KhaiiiApi
from konlpy.tag import Komoran
#import time

def Parser(parser = 'komoran'):

    return khaiiiParse() if parser == 'khaiii' else komoranParse()

class khaiiiParse:
    api = KhaiiiApi()

    def parse(self, text):
        #tic = time.perf_counter()
        exceptionTags = ['NNP','NP','VX']
        morphemes = []
        tags = []

        for entry in [x for x in self.api.analyze(text) if x.morphs[0].tag not in exceptionTags]:
            word = entry.morphs[0].lex
            tag = entry.morphs[0].tag
            if 'V' in tag[0]:
                word = word + '다'
            morphemes.append(word)
            tags.append(tag)

        sentence = {'morpheme': morphemes,
            'tag': tags}
        #toc = time.perf_counter()
        #print(toc-tic)
        return sentence
class komoranParse:
    api = Komoran()
    
    def parse(self, text):
        #tic = time.perf_counter()
        exceptionTags = ['JKB','ETM','JKO','JKG','JKS','EC','EP','EF','SF','SE','XSV','XPN','VX']
        morphemes = []
        tags = []

        for entry in [x for x in self.api.pos(text) if x[1] not in exceptionTags]:
            word,tag = entry
            if 'V' in tag:
                word = word + '다'
            morphemes.append(word)
            tags.append(tag)

        sentence = {'morpheme': morphemes,
            'tags': tags}
        #toc = time.perf_counter()
        #print(toc-tic)
        return sentence
