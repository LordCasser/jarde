public final class IterableForEach {
    public String test(Iterable<String> a) {
        StringBuilder sb = new StringBuilder();
        for (String s : a) {
            sb.append(s);
        }
        return sb.toString();
    }
}
