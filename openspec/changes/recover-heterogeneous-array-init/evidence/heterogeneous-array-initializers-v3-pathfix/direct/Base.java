public class Base implements LocalInterface {
  private final int value;

  public Base(int value) { this.value = value; }

  @Override
  public int value() { return value; }
}
