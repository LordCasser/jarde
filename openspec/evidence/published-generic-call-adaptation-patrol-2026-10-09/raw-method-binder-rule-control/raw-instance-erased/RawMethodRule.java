public class RawMethodRule<T> { public <U> U id(U x){return x;} public String use(RawMethodRule receiver, String x){return receiver.id(x);} }
