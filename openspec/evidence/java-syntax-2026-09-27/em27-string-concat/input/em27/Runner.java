package em27;

public final class Runner {
    private Runner() {}

    private static final class Mark {
        private final String text;

        Mark(String text) { this.text = text; }

        @Override public String toString() {
            System.out.print(text);
            return text;
        }
    }

    public static void main(String[] args) {
        System.out.println(Concat.builder(7));
        System.out.println(Concat.folded());
        System.out.println(Concat.objects(new Mark("A"), new Mark("B")));
        System.out.println(Concat.objects(null, null));
        System.out.println(Concat.character("name"));
        Concat.discarded(3);
        System.out.println(Concat.explicitConstructor());
        System.out.println(Concat.explicitConstructor() == "abc");
        System.out.println(Concat.explicitStored() == "abc");
    }
}
