#!/usr/bin/env python3
"""Synchronous stdlib expansion coverage; both custom optimization modes."""
import argparse,json,os,subprocess,sys,tempfile
from pathlib import Path

def main():
 parser=argparse.ArgumentParser(description=__doc__)
 parser.add_argument('--binary',type=Path,required=True);parser.add_argument('--output',type=Path)
 parser.add_argument('--case',action='append');parser.add_argument('--mode',choices=['plain','custom-O'])
 args=parser.parse_args();here=Path(__file__).resolve().parent;root=here.parents[1]
 cases=json.loads((here/'cases.json').read_text())
 if args.case:
  if set(args.case)-cases.keys():parser.error('unknown case')
  cases={k:v for k,v in cases.items() if k in args.case}
 for case in cases.values():
  if 'source_file' in case:case['source']=(here/case.pop('source_file')).read_text()
 with tempfile.TemporaryDirectory(prefix='fishyc-expansion-') as temporary:
  output=(args.output or Path(temporary)).resolve();output.mkdir(parents=True,exist_ok=True)
  manifest=output/'cases.json';manifest.write_text(json.dumps(cases,indent=2));results={}
  for mode in ([args.mode] if args.mode else ['plain','custom-O']):
   command=[sys.executable,str(root/'tests/stdlib/run.py'),'--binary',str(args.binary.resolve()),'--manifest',str(manifest),'--output',str(output/mode)]
   if mode=='custom-O':command.append('--custom-opt')
   subprocess.run(command,check=True)
   results.update({name+'/'+mode:r for name,r in json.loads((output/mode/'results.json').read_text()).items()})
  (output/'results.json').write_text(json.dumps(results,indent=2)+'\n')
  assert all(r['pass'] for r in results.values())
  print(f'{len(results)}/{len(results)} synchronous expansion configurations passed')
if __name__=='__main__':main()
