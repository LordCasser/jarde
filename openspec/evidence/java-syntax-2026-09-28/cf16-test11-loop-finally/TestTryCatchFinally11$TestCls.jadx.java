package jadx.tests.integration.trycatch;

import jadx.tests.api.utils.assertj.JadxAssertions;
import java.util.Arrays;
import java.util.List;

/* JADX INFO: loaded from: TestTryCatchFinally11$TestCls.class */
public class TestTryCatchFinally11$TestCls {
    private int count = 0;

    public void test(List<Object> list) {
        try {
            call1();
        } finally {
            for (Object item : list) {
                call2(item);
            }
        }
    }

    private void call1() {
        this.count += 100;
    }

    private void call2(Object item) {
        this.count++;
    }

    public void check() {
        TestTryCatchFinally11$TestCls t = new TestTryCatchFinally11$TestCls();
        t.test(Arrays.asList("1", "2"));
        JadxAssertions.assertThat(t.count).isEqualTo(102);
    }
}
