#!/usr/bin/env python3
"""Re-link captured objects with a test-only allocator guard."""
import json
from pathlib import Path
import subprocess
import tempfile
ROOT=Path(__file__).resolve().parents[1]

def main():
    result={}
    with tempfile.TemporaryDirectory(prefix='fishyc-guard-') as tmp:
        obj=Path(tmp)/'guard.o'
        subprocess.run(['cc','-c',str(ROOT/'tools/heap_guard.c'),'-o',str(obj)],check=True)
        for stage in ('before','after'):
            for name in ('padding2','aliasarc','selfassign','returnarc','closurefloat'):
                for mode in ('plain','custom-O'):
                    key=f'{stage}/{name}/{mode}'
                    d=ROOT/'compiler-review'/key
                    if not (d/'program.o').exists():continue
                    exe=Path(tmp)/'program'
                    subprocess.run(['cc',str(d/'program.o'),str(obj),'-Wl,--wrap=malloc','-Wl,--wrap=free','-o',str(exe)],check=True)
                    r=subprocess.run([str(exe)],capture_output=True,text=True,timeout=3)
                    result[key]={'exit':r.returncode,'stdout':r.stdout,'stderr':r.stderr}
    (ROOT/'compiler-review/heap-guard-results.json').write_text(json.dumps(result,indent=2)+'\n')
if __name__=='__main__':main()
