import java.util.Iterator;

final class IterableExceptionProbe {
    static String log = "";
    static int caught;
    static int finalized;

    static final class Boom extends RuntimeException { }

    static final class Source implements Iterable {
        final String fail;
        int index;
        Source(String fail) { this.fail = fail; }
        public Iterator iterator() {
            log += "I";
            if ("iterator".equals(fail)) throw new Boom();
            return new Iterator() {
                public boolean hasNext() {
                    log += "H";
                    if ("hasNext".equals(fail)) throw new Boom();
                    return index < 1;
                }
                public Object next() {
                    log += "N";
                    index++;
                    if ("next".equals(fail)) throw new Boom();
                    return "x";
                }
            };
        }
    }

    // One handler covers iterator(), hasNext(), next(), and the body. A finally records exit.
    static int uniform(Iterable values) {
        int total = 0;
        try {
            Iterator it = values.iterator();
            while (it.hasNext()) {
                Object value = it.next();
                total += value.toString().length();
            }
        } catch (Boom ex) {
            caught++;
            log += "C";
        } finally {
            finalized++;
            log += "F";
        }
        return total;
    }

    // iterator() has its own catch; hasNext()/next() share an outer catch/finally.
    static int iteratorBoundary(Iterable values) {
        log += "P"; // observable prefix before the iterator call
        Iterator it;
        try {
            it = values.iterator();
        } catch (Boom ex) {
            caught++;
            log += "Icatch";
            return -1;
        }
        int total = 0;
        try {
            while (it.hasNext()) {
                Object value = it.next();
                total += value.toString().length();
            }
        } catch (Boom ex) {
            caught++;
            log += "Lcatch";
        } finally {
            finalized++;
            log += "F";
        }
        return total;
    }

    // hasNext() has only the finally handler; next() also has the inner catch.
    static int nextBoundary(Iterable values) {
        log += "P";
        Iterator it = values.iterator();
        int total = 0;
        try {
            while (it.hasNext()) {
                try {
                    Object value = it.next();
                    total += value.toString().length();
                } catch (Boom ex) {
                    caught++;
                    log += "Ncatch";
                    break;
                }
            }
        } finally {
            finalized++;
            log += "F";
        }
        return total;
    }

    public static void main(String[] args) { }
}
