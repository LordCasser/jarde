package defpackage;

import java.util.Iterator;

/* JADX INFO: loaded from: probe-debug-final.jar:IterableExceptionProbe.class */
final class IterableExceptionProbe {
    static String log = "";
    static int caught;
    static int finalized;

    IterableExceptionProbe() {
    }

    /* JADX INFO: loaded from: probe-debug-final.jar:IterableExceptionProbe$Boom.class */
    static final class Boom extends RuntimeException {
        Boom() {
        }
    }

    /* JADX INFO: loaded from: probe-debug-final.jar:IterableExceptionProbe$Source.class */
    static final class Source implements Iterable {
        final String fail;
        int index;

        Source(String fail) {
            this.fail = fail;
        }

        @Override // java.lang.Iterable
        public Iterator iterator() {
            IterableExceptionProbe.log += "I";
            if ("iterator".equals(this.fail)) {
                throw new Boom();
            }
            return new Iterator() { // from class: IterableExceptionProbe.Source.1
                @Override // java.util.Iterator
                public boolean hasNext() {
                    IterableExceptionProbe.log += "H";
                    if ("hasNext".equals(Source.this.fail)) {
                        throw new Boom();
                    }
                    return Source.this.index < 1;
                }

                @Override // java.util.Iterator
                public Object next() {
                    IterableExceptionProbe.log += "N";
                    Source.this.index++;
                    if ("next".equals(Source.this.fail)) {
                        throw new Boom();
                    }
                    return "x";
                }
            };
        }
    }

    static int uniform(Iterable values) {
        int total = 0;
        try {
            for (Object value : values) {
                total += value.toString().length();
            }
        } catch (Boom e) {
            caught++;
            log += "C";
        } finally {
            finalized++;
            log += "F";
        }
        return total;
    }

    static int iteratorBoundary(Iterable values) {
        log += "P";
        try {
            int total = 0;
            for (Object value : values) {
                try {
                    try {
                        total += value.toString().length();
                    } catch (Boom e) {
                        caught++;
                        log += "Lcatch";
                        finalized++;
                        log += "F";
                    }
                } catch (Throwable th) {
                    finalized++;
                    log += "F";
                    throw th;
                }
            }
            finalized++;
            log += "F";
            return total;
        } catch (Boom e2) {
            caught++;
            log += "Icatch";
            return -1;
        }
    }

    static int nextBoundary(Iterable values) {
        log += "P";
        int total = 0;
        for (Object value : values) {
            try {
                try {
                    total += value.toString().length();
                } catch (Boom e) {
                    caught++;
                    log += "Ncatch";
                    finalized++;
                    log += "F";
                    return total;
                }
            } catch (Throwable th) {
                finalized++;
                log += "F";
                throw th;
            }
        }
        finalized++;
        log += "F";
        return total;
    }

    public static void main(String[] args) {
    }
}
