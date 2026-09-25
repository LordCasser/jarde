package defpackage;

import java.util.Iterator;

/* JADX INFO: loaded from: probe-nodebug-final.jar:IterableExceptionProbe.class */
final class IterableExceptionProbe {
    static String log = "";
    static int caught;
    static int finalized;

    /* JADX INFO: loaded from: probe-nodebug-final.jar:IterableExceptionProbe$Boom.class */
    static final class Boom extends RuntimeException {
        Boom() {
        }
    }

    /* JADX INFO: loaded from: probe-nodebug-final.jar:IterableExceptionProbe$Source.class */
    static final class Source implements Iterable {
        final String fail;
        int index;

        Source(String str) {
            this.fail = str;
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

    IterableExceptionProbe() {
    }

    static int uniform(Iterable iterable) {
        int length = 0;
        try {
            Iterator it = iterable.iterator();
            while (it.hasNext()) {
                length += it.next().toString().length();
            }
        } catch (Boom e) {
            caught++;
            log += "C";
        } finally {
            finalized++;
            log += "F";
        }
        return length;
    }

    static int iteratorBoundary(Iterable iterable) {
        log += "P";
        try {
            Iterator it = iterable.iterator();
            int length = 0;
            while (it.hasNext()) {
                try {
                    try {
                        length += it.next().toString().length();
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
            return length;
        } catch (Boom e2) {
            caught++;
            log += "Icatch";
            return -1;
        }
    }

    static int nextBoundary(Iterable iterable) {
        log += "P";
        Iterator it = iterable.iterator();
        int length = 0;
        while (it.hasNext()) {
            try {
                try {
                    length += it.next().toString().length();
                } catch (Boom e) {
                    caught++;
                    log += "Ncatch";
                    finalized++;
                    log += "F";
                    return length;
                }
            } catch (Throwable th) {
                finalized++;
                log += "F";
                throw th;
            }
        }
        finalized++;
        log += "F";
        return length;
    }

    public static void main(String[] strArr) {
    }
}
