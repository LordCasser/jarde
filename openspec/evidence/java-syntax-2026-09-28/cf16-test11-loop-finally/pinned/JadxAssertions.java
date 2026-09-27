package jadx.tests.api.utils.assertj;

// Compile-time stand-in for the assertion helper referenced by the full
// decompiled TestCls source. The replay runner exercises test(List) directly.
public final class JadxAssertions {
    public static Assertion assertThat(int value) {
        return new Assertion(value);
    }

    public static final class Assertion {
        private final int value;

        private Assertion(int value) {
            this.value = value;
        }

        public void isEqualTo(int expected) {
            if (value != expected) {
                throw new AssertionError("expected " + expected + ", got " + value);
            }
        }
    }
}
