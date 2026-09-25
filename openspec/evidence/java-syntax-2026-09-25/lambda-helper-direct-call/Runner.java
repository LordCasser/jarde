import java.util.function.IntUnaryOperator;

public final class Runner {
    public static void main(String[] args) {
        LambdaAlias instance = new LambdaAlias(5);
        IntUnaryOperator built = instance.build(7);
        System.out.println("lambda=" + built.applyAsInt(4));
        System.out.println("direct=" + instance.direct(2, 3));
    }
}
