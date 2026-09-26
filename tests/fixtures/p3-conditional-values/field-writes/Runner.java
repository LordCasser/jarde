public final class Runner {
    public static void main(String[] args) {
        ConditionalFieldWrites value = new ConditionalFieldWrites();
        for (boolean choose : new boolean[] { false, true }) {
            ConditionalFieldWrites.putStatic(choose);
            value.putInstance(choose);
            System.out.println(ConditionalFieldWrites.staticFlag + ":" + value.instanceFlag);
        }
    }
}
