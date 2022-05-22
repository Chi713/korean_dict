from typing import Optional
from konlpy.tag import Komoran

#optional khaiii package import
KhaiiiApi: Optional[type] = None
try:
    #import obviously_not_real_package_used_for_testing
    from khaiii import KhaiiiApi 
except ImportError as err:
    print(f"module {err} not installed.\nUsing default parser")
    pass

def Parser(parser: str = 'komoran'):

    if parser == 'khaiii' and KhaiiiApi != None:
        print('using khaiii')
        return khaiiiParse()
    else:
        print("using komoran")
        return komoranParse()

#class is under if statement to satisfy static type checking and TypeError being thrown at runtime
if KhaiiiApi != None:
    class khaiiiParse:
        api = KhaiiiApi()

        def parse(self, text: str):
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
    
    def parse(self, text: str):
        exceptionTags = ['JKB','ETM','JKO','JKG','JKS','EC','EP','EF','SF','SE','XSV','XPN','VX']
        morphemes = []
        tags = []

        for entry in [x for x in self.api.pos(text) if x[1] not in exceptionTags]:
            word,tag = entry
            if 'V' in tag:
                word = word + '다'
            morphemes.append(word)
            tags.append(tag)

        sentence: dict = {'morpheme': morphemes,
            'tags': tags}
        return sentence
