import static jadx.tests.api.utils.assertj.JadxAssertions.assertThat;

public class TestTryCatchFinallyError {
  public static class TestCls {
    public boolean f;

    private boolean test(Object obj) {
      this.f = false;
      try {
        exc(obj);
      } catch (Exception e) {
        e.printStackTrace();
      } finally {
        this.f = true;
      }
      return this.f;
    }

    private static boolean exc(Object obj) throws Exception {
      if (obj instanceof Error) throw (Error) obj;
      if (obj == null) throw new Exception("test");
      return obj instanceof String;
    }

    public void check() {
      assertThat(test("a")).isTrue();
      assertThat(test(null)).isTrue();
    }
  }
}
