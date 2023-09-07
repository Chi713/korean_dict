import sys
import json
from khaiii import KhaiiiApi
    
words_list = list()
for word in [w.morphs for w in KhaiiiApi().analyze(sys.argv[1])]:
    words_list.append([(mor.lex, mor.tag) for mor in word])

word_list_json = json.dumps(words_list, indent=4, ensure_ascii=False)
print(word_list_json)
