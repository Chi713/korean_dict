import csv
import pprint
from pathlib import Path

class csvParse:

    def parse(self, 
              file, 
              fields = ['tag', 'seq_marker', 'audio', 'picture', 'TL', 'NL']): 

        PROJECT_ROOT = Path(__file__).parents[2]
        output = dict()

        with open(PROJECT_ROOT / file) as f:
            tsv_file = csv.reader(f, delimiter='\t')
            #fields = ['tag', 'seq_marker', 'audio', 'picture', 'TL', 'NL']
            for (index, line) in enumerate(tsv_file):
                #dict_line = {fields[i]: line for i in range(len(line))}
                #print(i for i in reversed(range(len(line))))
                output[index] = {fields[i]: line[i] for i in range(len(line))}

            #pprint.pprint(output)

        #type is '_csv.reader'
        #return tsv_file

        return output

test_file = "resources/bite_sisters.tsv"

pp= pprint.PrettyPrinter(compact=True, sort_dicts=False)
cParse = csvParse()
result = cParse.parse(test_file)
#print(result[1])
pp.pprint(result[1]['TL'])
