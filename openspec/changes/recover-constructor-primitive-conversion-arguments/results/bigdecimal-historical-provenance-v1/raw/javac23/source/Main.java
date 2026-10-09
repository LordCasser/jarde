import java.math.BigDecimal;
public class Main {
  public static void main(String[] args) {
    Number[] values = new Number[]{new BigDecimal("1.25")};
    System.out.println(values.length + ":" + values[0]);
  }
}
