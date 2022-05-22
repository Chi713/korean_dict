import csv
import pprint
from pathlib import Path

class csvParse:

    #def __init__(self, file) -> None:
        

    def parse(self, file): 

        PROJECT_ROOT = Path(__file__).parents[2]
        output = list()

        with open(PROJECT_ROOT / file) as f:
            tsv_file = csv.reader(f, delimiter='\t')
            for line in tsv_file:
                output.append(line)

            #pprint.pprint(output)

        #type is '_csv.reader'
        #return tsv_file

        return output

test_file = "resources/bite_sisters.tsv"

pp= pprint.PrettyPrinter(compact=True)
cParse = csvParse()
result = cParse.parse(test_file)
pp.pprint(result[1][4])
