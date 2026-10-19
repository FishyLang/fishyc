#!/usr/bin/env python3
"""Compare captured baseline cases; do not use baseline behavior as an oracle."""
import difflib
import hashlib
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]/'compiler-review'

def main():
    names=json.loads((ROOT/'baseline-cases.json').read_text())
    summary={}
    for name in names:
        for mode in ('plain','custom-O'):
            key=f'{name}/{mode}'
            before,after=(ROOT/stage/key for stage in ('before','after'))
            assert (before/'source.fih').read_bytes()==(after/'source.fih').read_bytes(),key
            result={}
            for artifact in ('custom-ir.txt','llvm-unoptimized.ll','llvm-optimized.ll'):
                a,b=before/artifact,after/artifact
                entry={'before_exists':a.exists(),'after_exists':b.exists()}
                for stage,path in [('before',a),('after',b)]:
                    if path.exists():
                        entry[stage+'_lines']=len(path.read_text().splitlines())
                        entry[stage+'_sha256']=hashlib.sha256(path.read_bytes()).hexdigest()
                if a.exists() and b.exists():
                    entry['identical']=a.read_bytes()==b.read_bytes()
                    diff=difflib.unified_diff(a.read_text().splitlines(True),b.read_text().splitlines(True),fromfile='before/'+artifact,tofile='after/'+artifact)
                    (after/(artifact+'.diff')).write_text(''.join(diff))
                result[artifact]=entry
            summary[key]=result
    (ROOT/'ir-comparison.json').write_text(json.dumps(summary,indent=2)+'\n')
    print(f'Compared {len(summary)} unchanged source configurations.')
if __name__=='__main__':main()
