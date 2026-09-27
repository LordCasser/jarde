package jadx.tests.integration.trycatch;

import jadx.tests.api.utils.assertj.JadxAssertions;

/* JADX INFO: loaded from: TestTryCatchFinally12$TestCls.class */
public class TestTryCatchFinally12$TestCls {
    private StringBuilder sb;

    public void test1(int i) {
        try {
            try {
                call(i);
            } catch (NullPointerException e) {
                this.sb.append("-catch");
            }
            this.sb.append("-out");
        } finally {
            this.sb.append("-finally");
        }
    }

    public void test2(int i) {
        try {
            try {
                call(i);
            } catch (NullPointerException e) {
                this.sb.append("-catch");
            }
        } finally {
            this.sb.append("-finally");
        }
    }

    public void test3(int i) {
        try {
            call(i);
        } catch (NullPointerException e) {
            this.sb.append("-catch");
        } finally {
            this.sb.append("-finally");
        }
    }

    public void call(int i) {
        this.sb.append("call");
        switch (i) {
            case 1:
                this.sb.append("-npe");
                throw new NullPointerException();
            case 2:
                this.sb.append("-iae");
                throw new IllegalArgumentException();
            default:
                return;
        }
    }

    public String runTest(int i, int i2) {
        this.sb = new StringBuilder();
        try {
            switch (i) {
                case 1:
                    test1(i2);
                    break;
                case 2:
                    test2(i2);
                    break;
                case 3:
                    test3(i2);
                    break;
            }
        } catch (IllegalArgumentException e) {
            JadxAssertions.assertThat(Integer.valueOf(i2)).isEqualTo(2);
        }
        return this.sb.toString();
    }

    public void check() {
        JadxAssertions.assertThat(runTest(1, 0)).isEqualTo("call-out-finally");
        JadxAssertions.assertThat(runTest(1, 1)).isEqualTo("call-npe-catch-out-finally");
        JadxAssertions.assertThat(runTest(1, 2)).isEqualTo("call-iae-finally");
        JadxAssertions.assertThat(runTest(2, 0)).isEqualTo("call-finally");
        JadxAssertions.assertThat(runTest(2, 1)).isEqualTo("call-npe-catch-finally");
        JadxAssertions.assertThat(runTest(2, 2)).isEqualTo("call-iae-finally");
        JadxAssertions.assertThat(runTest(3, 0)).isEqualTo("call-finally");
        JadxAssertions.assertThat(runTest(3, 1)).isEqualTo("call-npe-catch-finally");
        JadxAssertions.assertThat(runTest(3, 2)).isEqualTo("call-iae-finally");
    }
}
