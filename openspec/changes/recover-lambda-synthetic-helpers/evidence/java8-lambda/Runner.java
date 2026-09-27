public final class Runner {
    public static void main(String[] args) {
        System.out.println(LambdaFixture.zero().getAsInt());
        System.out.println(LambdaFixture.one().applyAsInt(5));
        System.out.println(LambdaFixture.two().applyAsInt(4, 2));
    }
}
