import java.util.function.IntUnaryOperator;
public class IntBox {
    public final IntUnaryOperator op;
    public IntBox(IntUnaryOperator op) { this.op = op; }
}
