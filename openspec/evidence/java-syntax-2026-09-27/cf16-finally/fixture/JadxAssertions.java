package jadx.tests.api.utils.assertj;
public final class JadxAssertions {
  private JadxAssertions() {}
  public static BoolAssertion assertThat(boolean value) { return new BoolAssertion(value); }
  public static final class BoolAssertion {
    private final boolean value;
    BoolAssertion(boolean value) { this.value = value; }
    public void isTrue() { if (!value) throw new AssertionError("expected true"); }
  }
}
