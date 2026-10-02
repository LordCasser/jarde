package defpackage;

import java.util.Arrays;

/* JADX INFO: loaded from: W17c.class */
public class W17c implements AutoCloseable {
    static StringBuilder log = new StringBuilder();
    static boolean closeBoom = false;

    @Override // java.lang.AutoCloseable
    public void close() {
        if (closeBoom) {
            throw new IllegalStateException("close");
        }
    }

    static void touch(W17c w17c) {
    }

    static void boom() {
        throw new IllegalStateException("body");
    }

    public static String normalReturn() {
        try {
            W17c w17c = new W17c();
            try {
                touch(w17c);
                w17c.close();
                return "done";
            } catch (Throwable th) {
                try {
                    w17c.close();
                } catch (Throwable th2) {
                    th.addSuppressed(th2);
                }
                throw th;
            }
        } catch (IllegalStateException e) {
            return "caught";
        }
    }

    public static String callCatch() {
        try {
            W17c w17c = new W17c();
            try {
                touch(w17c);
                w17c.close();
                return "done";
            } catch (Throwable th) {
                try {
                    w17c.close();
                } catch (Throwable th2) {
                    th.addSuppressed(th2);
                }
                throw th;
            }
        } catch (IllegalStateException e) {
            log.append("E");
            return "done";
        }
    }

    public static String bodyThrows() {
        try {
            W17c w17c = new W17c();
            try {
                boom();
                w17c.close();
                return "done";
            } catch (Throwable th) {
                try {
                    w17c.close();
                } catch (Throwable th2) {
                    th.addSuppressed(th2);
                }
                throw th;
            }
        } catch (IllegalStateException e) {
            log.append("E");
            return "done";
        }
    }

    public static String callThenReturn() {
        try {
            W17c w17c = new W17c();
            try {
                boom();
                w17c.close();
                return "done";
            } catch (Throwable th) {
                try {
                    w17c.close();
                } catch (Throwable th2) {
                    th.addSuppressed(th2);
                }
                throw th;
            }
        } catch (IllegalStateException e) {
            log.append("E");
            return "caught";
        }
    }

    public static String twoCalls() {
        try {
            W17c w17c = new W17c();
            try {
                boom();
                w17c.close();
                return "done";
            } catch (Throwable th) {
                try {
                    w17c.close();
                } catch (Throwable th2) {
                    th.addSuppressed(th2);
                }
                throw th;
            }
        } catch (IllegalStateException e) {
            log.append("a");
            log.append("b");
            return "done";
        }
    }

    public static String chainedConsume() {
        try {
            W17c w17c = new W17c();
            try {
                boom();
                w17c.close();
                return "done";
            } catch (Throwable th) {
                try {
                    w17c.close();
                } catch (Throwable th2) {
                    th.addSuppressed(th2);
                }
                throw th;
            }
        } catch (IllegalStateException e) {
            log.append(e.getMessage());
            return "done";
        }
    }

    public static String branchBody() {
        try {
            W17c w17c = new W17c();
            try {
                boom();
                w17c.close();
                return "done";
            } catch (Throwable th) {
                try {
                    w17c.close();
                } catch (Throwable th2) {
                    th.addSuppressed(th2);
                }
                throw th;
            }
        } catch (IllegalStateException e) {
            log.append("B");
            if (closeBoom) {
                log.append("1");
                return "done";
            }
            log.append("2");
            return "done";
        }
    }

    public static String closeThrows() {
        try {
            W17c w17c = new W17c();
            try {
                touch(w17c);
                w17c.close();
                return "done";
            } catch (Throwable th) {
                try {
                    w17c.close();
                } catch (Throwable th2) {
                    th.addSuppressed(th2);
                }
                throw th;
            }
        } catch (IllegalStateException e) {
            log.append("C");
            return "done";
        }
    }

    public static String suppressedBoth() {
        try {
            W17c w17c = new W17c();
            try {
                boom();
                w17c.close();
                return "done";
            } catch (Throwable th) {
                try {
                    w17c.close();
                } catch (Throwable th2) {
                    th.addSuppressed(th2);
                }
                throw th;
            }
        } catch (IllegalStateException e) {
            log.append("[" + Arrays.toString(e.getSuppressed()) + "]");
            return "done";
        }
    }

    public static void main(String[] strArr) {
        System.out.println(normalReturn());
        System.out.println(callCatch());
        System.out.println(bodyThrows());
        System.out.println(callThenReturn());
        System.out.println(twoCalls());
        System.out.println(chainedConsume());
        closeBoom = true;
        System.out.println(closeThrows());
        System.out.println(suppressedBoth());
        closeBoom = false;
        System.out.println(branchBody());
        System.out.println("log:" + ((Object) log));
    }
}
