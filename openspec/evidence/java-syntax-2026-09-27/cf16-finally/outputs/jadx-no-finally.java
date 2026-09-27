package defpackage;

import jadx.tests.api.utils.assertj.JadxAssertions;

/* JADX INFO: loaded from: TestTryCatchFinally$TestCls.class */
public class TestTryCatchFinally$TestCls {
    public boolean f;

    private boolean test(Object obj) {
        this.f = false;
        try {
            try {
                exc(obj);
                this.f = true;
            } catch (Exception e) {
                e.printStackTrace();
                this.f = true;
            }
            return this.f;
        } catch (Throwable th) {
            this.f = true;
            throw th;
        }
    }

    private static boolean exc(Object obj) throws Exception {
        if (obj == null) {
            throw new Exception("test");
        }
        return obj instanceof String;
    }

    public void check() {
        JadxAssertions.assertThat(test("a")).isTrue();
        JadxAssertions.assertThat(test(null)).isTrue();
    }
}
