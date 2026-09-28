package jadx.tests.integration.trycatch;

import java.util.ArrayList;
import java.util.List;

/* JADX INFO: loaded from: TestTryCatchFinally5$TestCls.class */
public class TestTryCatchFinally5$TestCls {

    /* JADX INFO: loaded from: TestTryCatchFinally5$TestCls$A.class */
    private interface A {
    }

    /* JADX INFO: loaded from: TestTryCatchFinally5$TestCls$B.class */
    private interface B<T> {
        D f(C c);

        T load(D d);
    }

    /* JADX INFO: loaded from: TestTryCatchFinally5$TestCls$C.class */
    private interface C {
    }

    /* JADX INFO: loaded from: TestTryCatchFinally5$TestCls$D.class */
    private interface D {
        boolean first();

        boolean toNext();

        void close();
    }

    public <E> List<E> test(A a, B<E> b) {
        C c = p(a);
        if (c == null) {
            return null;
        }
        D d = b.f(c);
        try {
            if (!d.first()) {
                return null;
            }
            List<E> list = new ArrayList<>();
            do {
                list.add(b.load(d));
            } while (d.toNext());
            return list;
        } finally {
            d.close();
        }
    }

    private C p(A a) {
        return (C) a;
    }
}
