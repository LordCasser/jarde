package defpackage;

import java.io.BufferedReader;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.text.ParseException;
import java.util.ArrayList;
import java.util.List;
import java.util.Objects;
import java.util.function.Function;
import java.util.function.Supplier;

/* JADX INFO: loaded from: Patrol.class */
public class Patrol {
    static int staticRef(String s) {
        return s.length();
    }

    int boundRef() {
        return 42;
    }

    static int anyRef(Patrol p, String s) {
        return s.hashCode() + p.boundRef();
    }

    Supplier<List<String>> listCtor() {
        return ArrayList::new;
    }

    Function<Integer, int[]> arrayCtor() {
        return x$0 -> {
            return new int[x$0];
        };
    }

    int dispatch(Supplier<Integer> s, Function<String, Integer> f) {
        return s.get().intValue() + f.apply("x").intValue();
    }

    int supply() {
        return 7;
    }

    int exerciseMethodRefs(Patrol other) {
        int iDispatch = dispatch(this::supply, Patrol::staticRef);
        Objects.requireNonNull(other);
        return iDispatch + dispatch(other::boundRef, this::len) + listCtor().get().size() + arrayCtor().apply(3).length;
    }

    int len(String s) {
        return s.length();
    }

    void preciseRethrow(String mode) throws Exception {
        try {
            if (mode.equals("parse")) {
                throw new ParseException("p", 0);
            }
            if (mode.equals("io")) {
                throw new IOException("io");
            }
        } catch (Exception e) {
            log(e);
            throw e;
        }
    }

    static void log(Exception e) {
    }

    int labeledBlock(int arg0) {
        int r = 0;
        if (arg0 >= 0) {
            int r2 = arg0 * 2;
            r = arg0 == 0 ? 1 : r2 + 3;
        }
        return r;
    }

    int multiResource(Path a, Path b) throws IOException {
        BufferedReader r1 = Files.newBufferedReader(a);
        try {
            BufferedReader r2 = Files.newBufferedReader(b);
            try {
                int i = r1.read() + r2.read();
                if (r2 != null) {
                    r2.close();
                }
                if (r1 != null) {
                    r1.close();
                }
                return i;
            } catch (Throwable th) {
                if (r2 != null) {
                    try {
                        r2.close();
                    } catch (Throwable th2) {
                        th.addSuppressed(th2);
                    }
                }
                throw th;
            }
        } catch (Throwable th3) {
            if (r1 != null) {
                try {
                    r1.close();
                } catch (Throwable th4) {
                    th3.addSuppressed(th4);
                }
            }
            throw th3;
        }
    }

    int charOps(char c, int arg1) {
        int r;
        char up = (char) (c - ' ');
        int r2 = up + arg1;
        switch (up) {
            case 'A':
                r = r2 + 1;
                break;
            case 'B':
                r = r2 + 2;
                break;
            default:
                r = r2 + 3;
                break;
        }
        return r;
    }

    int forever(int arg0) {
        int n = 0;
        while (true) {
            n++;
            int i = 0;
            while (true) {
                if (i < 3) {
                    if (i != 1) {
                        if (i == 2) {
                            break;
                        }
                        if (n <= 10) {
                            n += i;
                        } else {
                            return n;
                        }
                    }
                    i++;
                } else {
                    n += 100;
                    if (n <= 1000) {
                        break;
                    }
                    return n;
                }
            }
        }
    }

    int unreachableAfter(int arg0) {
        int n = 0;
        while (true) {
            n++;
            int i = 0;
            while (true) {
                if (i < 3) {
                    if (i != 1) {
                        if (i == 2) {
                            break;
                        }
                        if (n <= 10) {
                            n += i;
                        } else {
                            return n;
                        }
                    }
                    i++;
                } else {
                    n += 100;
                    if (n <= 1000) {
                        break;
                    }
                    return -n;
                }
            }
        }
    }
}
