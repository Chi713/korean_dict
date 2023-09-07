import sys
import json
from konlpy.tag import Komoran

print(json.dumps(Komoran().pos(sys.argv[1]), indent=4, ensure_ascii=False))
