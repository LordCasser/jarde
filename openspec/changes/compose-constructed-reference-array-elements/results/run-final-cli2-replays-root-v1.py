from pathlib import Path
import subprocess,sys
r=Path(__file__).resolve().parent
commands=[['replay-complete-family-root-v1.py','--label','candidate-cli2-complete-family-v1','--cli','/private/tmp/jarde-em18-composition-cli-v2'],['verify-family-root-v1.py','candidate-cli2-complete-family-v1'],['composition-replay-baseline18-root-v4.py'],['composition-replay-fixture-root-v4.py'],['replay-nested-controls-root-v2.py']]
for args in commands:
 print('START',args[0],flush=True)
 subprocess.run([sys.executable,str(r/args[0]),*args[1:]],check=True)
