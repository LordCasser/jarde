public final class Runner {
    public static void main(String[] args) {
        if (args[0].equals("single")) {
            System.out.println(single.Subject.select(single.Mode.ONE));
            System.out.println(single.Subject.select(single.Mode.TWO));
            System.out.println(single.Subject.select(single.Mode.THREE));
            try { single.Subject.select(null); throw new AssertionError("null accepted"); }
            catch (NullPointerException expected) { System.out.println("null:NPE"); }
        } else {
            System.out.println(doublecase.Subject.select(doublecase.Count.ONE, doublecase.Animal.DOG));
            System.out.println(doublecase.Subject.select(doublecase.Count.TWO, doublecase.Animal.CAT));
            System.out.println(doublecase.Subject.select(doublecase.Count.THREE, doublecase.Animal.DOG));
            try { doublecase.Subject.select(null, doublecase.Animal.CAT); throw new AssertionError("null count accepted"); }
            catch (NullPointerException expected) { System.out.println("null-count:NPE"); }
            try { doublecase.Subject.select(doublecase.Count.ONE, null); throw new AssertionError("null animal accepted"); }
            catch (NullPointerException expected) { System.out.println("null-animal:NPE"); }
        }
    }
}
