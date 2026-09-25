# Synchronized multiple exits

Source-only Java 8 audit of two normal return paths from one synchronized body and an exception from one value producer. See `analysis.md` for bytecode and current guard/Plan/Region boundaries. Run `python3 run_audit.py` to replay; no generated class source is edited.
