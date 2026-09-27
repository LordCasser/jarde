public final class Runner {
    private static void run(String label, grouped.Count count, grouped.Animal animal) {
        grouped.Subject.trace = 0;
        try {
            System.out.println(label + "=" + grouped.Subject.select(count, animal)
                    + ":trace=" + grouped.Subject.trace);
        } catch (NullPointerException expected) {
            System.out.println(label + "=NPE:trace=" + grouped.Subject.trace);
        }
    }

    public static void main(String[] args) {
        run("one-cat", grouped.Count.ONE, grouped.Animal.CAT);
        run("one-dog", grouped.Count.ONE, grouped.Animal.DOG);
        run("two-cat", grouped.Count.TWO, grouped.Animal.CAT);
        run("two-dog", grouped.Count.TWO, grouped.Animal.DOG);
        run("three-cat", grouped.Count.THREE, grouped.Animal.CAT);
        run("three-dog", grouped.Count.THREE, grouped.Animal.DOG);
        run("null-count", null, grouped.Animal.CAT);
        run("null-animal", grouped.Count.ONE, null);
    }
}
