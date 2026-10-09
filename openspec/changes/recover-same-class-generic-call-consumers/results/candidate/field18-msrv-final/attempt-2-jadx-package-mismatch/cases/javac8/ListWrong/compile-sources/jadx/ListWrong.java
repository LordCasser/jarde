package defpackage;

import java.util.List;

/* JADX INFO: loaded from: ListWrong.jar:ListWrong.class */
public class ListWrong<T> {
    public List<T> v;

    /* JADX WARN: Multi-variable type inference failed */
    public void put(List<String> list) {
        this.v = list;
    }
}
