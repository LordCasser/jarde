// A stand-in for the JDK interface: the same binary name, the same erased member, compiled from
// readable source so a test can place a *provided* java/lang/Comparable in a snapshot.
public interface Comparable {
    int compareTo(Object o);
}
