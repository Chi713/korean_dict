from khaiii import KhaiiiApi
from konlpy.tag import Komoran
def Parser(parser = 'komoran'):

    if parser == 'khaiii':
        return khaiiiParse()
    else: 
        return komoranParse()

class khaiiiParse:
    api = KhaiiiApi()
    def parse(self, text):
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

        return sentence
class komoranParse:
    api = Komoran()
    def parse(self, text):
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

        return sentence
