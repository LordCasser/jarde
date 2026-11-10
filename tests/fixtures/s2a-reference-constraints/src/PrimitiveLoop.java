public final class PrimitiveLoop {
    public static long value(int rounds) {
        long total = 0;
        while (rounds-- > 0) {
            { int item = rounds + 2; total += item; }
            { long item = rounds + 3L; total += item; }
            { double item = rounds + 4.0; total += (long) item; }
        }
        return total;
    }
}
