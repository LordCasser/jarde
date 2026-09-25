public final class FloatingBoundaryRunner {
    public static void main(String[] args) {
        Class<?> type = FloatingBoundary.class;
        for (String name : new String[] {"negativeZero", "leastSubnormal", "greatestFinite", "negativeInfinity"}) {
            try {
                Object value = type.getDeclaredMethod(name).getDefaultValue();
                System.out.printf("%s=%08x%n", name, Float.floatToRawIntBits((Float) value));
            } catch (Exception e) { throw new RuntimeException(e); }
        }
        for (String name : new String[] {"leastSubnormalDouble", "greatestFiniteDouble", "positiveInfinity", "canonicalNaN"}) {
            try {
                Object value = type.getDeclaredMethod(name).getDefaultValue();
                System.out.printf("%s=%016x%n", name, Double.doubleToRawLongBits((Double) value));
            } catch (Exception e) { throw new RuntimeException(e); }
        }
    }
}
