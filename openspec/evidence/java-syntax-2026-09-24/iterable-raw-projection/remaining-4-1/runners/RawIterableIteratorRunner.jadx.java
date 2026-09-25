package defpackage;
import java.util.Arrays;
import java.util.Iterator;

public final class RawIterableIteratorRunner {
    public static void main(String[] args) {
        Iterable values = Arrays.asList("a", "bc");
        System.out.println("raw=" + RawIterableIterator.rawCast(values));
        RawIterableIterator.touches = 0;
        System.out.println("castBeforeTouch="
                + RawIterableIterator.nextTwiceCastBeforeTouch(values)
                + ",touches=" + RawIterableIterator.touches);
        RawIterableIterator.touches = 0;
        System.out.println("touchBeforeCast="
                + RawIterableIterator.nextTwiceTouchBeforeCast(values)
                + ",touches=" + RawIterableIterator.touches);

        Iterable wrong = Arrays.asList(Integer.valueOf(7));
        RawIterableIterator.touches = 0;
        try {
            RawIterableIterator.nextTwiceCastBeforeTouch(wrong);
            System.out.println("wrongCastBeforeTouch=none");
        } catch (RuntimeException e) {
            System.out.println("wrongCastBeforeTouch=" + e.getClass().getSimpleName()
                    + ",touches=" + RawIterableIterator.touches);
        }
        RawIterableIterator.touches = 0;
        try {
            RawIterableIterator.nextTwiceTouchBeforeCast(wrong);
            System.out.println("wrongTouchBeforeCast=none");
        } catch (RuntimeException e) {
            System.out.println("wrongTouchBeforeCast=" + e.getClass().getSimpleName()
                    + ",touches=" + RawIterableIterator.touches);
        }

        Iterable throwsInNext = new Iterable() {
            public Iterator iterator() {
                return new Iterator() {
                    public boolean hasNext() { return true; }
                    public Object next() { throw new IllegalStateException("next"); }
                    public void remove() { throw new UnsupportedOperationException(); }
                };
            }
        };
        RawIterableIterator.touches = 0;
        try {
            RawIterableIterator.touchBeforeNext(throwsInNext);
            System.out.println("nextFailure=none");
        } catch (RuntimeException e) {
            System.out.println("nextFailure=" + e.getClass().getSimpleName()
                    + ",touches=" + RawIterableIterator.touches);
        }
    }
}
