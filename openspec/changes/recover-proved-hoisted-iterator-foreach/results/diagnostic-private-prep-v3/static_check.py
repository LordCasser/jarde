from pathlib import Path
import hashlib, json
p=Path('/private/tmp/jarde-hoisted-iterator-diagnostic-prep-luna-v3/build.rs.candidate')
s=p.read_text()
# A small lexical delimiter scan that skips Rust line/block comments and quoted strings/chars.
stack=[]; i=0; pairs={')':'(',']':'[','}':'{'}; calls=0; call_arities=[]
while i < len(s):
    if s.startswith('//', i):
        j=s.find('\n', i); i=len(s) if j<0 else j+1; continue
    if s.startswith('/*', i):
        depth=1; i+=2
        while i<len(s) and depth:
            if s.startswith('/*',i): depth+=1; i+=2
            elif s.startswith('*/',i): depth-=1; i+=2
            else: i+=1
        if depth: raise SystemExit('unterminated block comment')
        continue
    # Rust raw strings r###" ... "###
    if s[i]=='r':
        j=i+1
        while j<len(s) and s[j]=='#': j+=1
        if j<len(s) and s[j]=='"':
            end='"' + '#'*(j-i-1); k=s.find(end,j+1)
            if k<0: raise SystemExit('unterminated raw string')
            i=k+len(end); continue
    if s[i] in ('"', "'"):
        q=s[i]
        # Apostrophe lifetime is not a character string unless a closing quote is nearby.
        if q=="'":
            k=i+1; esc=False; found=False
            while k<min(len(s),i+8) and s[k]!='\n':
                if not esc and s[k]=="'": found=True; break
                if not esc and s[k]=='\\': esc=True
                else: esc=False
                k+=1
            if not found: i+=1; continue
        i+=1; esc=False
        while i<len(s):
            c=s[i]
            if not esc and c==q: i+=1; break
            if not esc and c=='\\': esc=True
            else: esc=False
            i+=1
        continue
    if s.startswith('log_foreach_candidate(',i):
        calls+=1; j=i+len('log_foreach_candidate('); d=1; commas=0; nonempty=False; in_string=False; esc=False
        while j<len(s) and d:
            c=s[j]
            if in_string:
                if not esc and c=='"': in_string=False
                if not esc and c=='\\': esc=True
                else: esc=False
            elif c=='"': in_string=True
            elif c=='(': d+=1
            elif c==')': d-=1
            elif c=='[': pass
            elif c==',' and d==1: commas+=1
            elif d==1 and not c.isspace(): nonempty=True
            j+=1
        # Includes the first argument before comma; four arguments have three top-level commas.
        
        k=j-2
        while k>=0 and s[k].isspace(): k-=1
        trailing=1 if k>=0 and s[k]==',' else 0
        call_arities.append(commas+1-trailing)
    c=s[i]
    if c in '([{': stack.append((c,i))
    elif c in ')]}':
        if not stack or stack[-1][0]!=pairs[c]: raise SystemExit(f'delimiter mismatch at {i}')
        stack.pop()
    i+=1
if stack: raise SystemExit(f'unclosed delimiters {stack[-3:]}')
if any(n!=4 for n in call_arities): raise SystemExit(f'logger call arity mismatch {call_arities}')
print(f'lexical delimiter scan passed; logger calls={calls}, each has 4 arguments')
# Verify the targeted observation and its logger share the lexical diagnostic block.
def matching_brace(source, open_at):
    depth=0; i=open_at
    while i<len(source):
        if source.startswith('//',i):
            j=source.find('\n',i); i=len(source) if j<0 else j+1; continue
        if source.startswith('/*',i):
            n=1; i+=2
            while i<len(source) and n:
                if source.startswith('/*',i): n+=1; i+=2
                elif source.startswith('*/',i): n-=1; i+=2
                else: i+=1
            if n: raise SystemExit('unterminated block comment while checking scope')
            continue
        if source[i]=='r':
            j=i+1
            while j<len(source) and source[j]=='#': j+=1
            if j<len(source) and source[j]=='"':
                end='"'+'#'*(j-i-1); k=source.find(end,j+1)
                if k<0: raise SystemExit('unterminated raw string while checking scope')
                i=k+len(end); continue
        if source[i] in ('"', "'"):
            q=source[i]
            if q=="'":
                k=i+1; esc=False; found=False
                while k<min(len(source),i+8) and source[k]!='\n':
                    if not esc and source[k]=="'": found=True; break
                    if not esc and source[k]=='\\': esc=True
                    else: esc=False
                    k+=1
                if not found: i+=1; continue
            i+=1; esc=False
            while i<len(source):
                c=source[i]
                if not esc and c==q: i+=1; break
                if not esc and c=='\\': esc=True
                else: esc=False
                i+=1
            continue
        if source[i]=='{': depth+=1
        elif source[i]=='}':
            depth-=1
            if depth==0: return i
        i+=1
    raise SystemExit('unclosed diagnostic scope block')
trace_pos=s.index('let mut phi_trace_values')
logger_pos=s.index('"phi_use_expansion"')
if_scope=s.rfind('if diagnostic_scope {',0,trace_pos)
if if_scope<0: raise SystemExit('diagnostic_scope block not found before trace variables')
open_at=s.index('{',if_scope)
close_at=matching_brace(s,open_at)
if not (if_scope<trace_pos<logger_pos<close_at):
    raise SystemExit('phi trace variables and logger are not in the same diagnostic_scope block')
print('phi trace variable/logger lexical scope check passed')
