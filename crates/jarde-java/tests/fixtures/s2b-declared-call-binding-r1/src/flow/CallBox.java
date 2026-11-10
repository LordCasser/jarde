package flow;

import java.util.List;

public class CallBox<T> {
    public T get() { return null; }
    public List<String> fixed() { return null; }
    public List<String> fixed(String left, String right) { return null; }
    public void set(T value) {}
    public void set(String value) {}
    public void set(int left, int right) {}
    public <U> U infer(U value) { return value; }
}

class VarargBox {
    public void run(String value) {}
    public void run(String... values) {}
}

class BridgeBase<T> {
    public T echo(T value) { return value; }
}

class BridgeChild extends BridgeBase<String> {
    @Override public String echo(String value) { return value; }
}

class KeepBase {
    public void keep(String value) {}
}

class KeepChild extends KeepBase {
    @Override public void keep(String value) {}
}
