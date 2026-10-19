import os, subprocess, sys
seconds=float(sys.argv[1]); cwd=os.path.realpath(sys.argv[2]); argv=sys.argv[3:]
if not argv: raise SystemExit("runtime wrapper requires a child argv")
p=subprocess.Popen(argv,cwd=cwd)
try: sys.exit(p.wait(timeout=seconds))
except subprocess.TimeoutExpired:
 p.kill(); p.wait(); print("ROOT_RUNTIME_TIMEOUT_SECONDS="+str(seconds),file=sys.stderr); sys.exit(124)
