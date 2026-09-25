import java.util.function.IntUnaryOperator;
public class LambdaCapture {
 public static IntUnaryOperator create(){int base=CaptureSupport.next();return value->base+value;}
}
