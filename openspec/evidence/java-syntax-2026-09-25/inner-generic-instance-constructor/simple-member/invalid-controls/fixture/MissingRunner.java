package nested;

public final class MissingRunner {
    public static void main(String[] args) {
        try {
            Object result = UseInner.make(new SimpleOuter(4), 3);
            System.out.println("made:" + result.getClass().getName());
        } catch (NoClassDefFoundError failure) {
            System.out.println("missing:" + failure.getMessage());
        }
    }
}
