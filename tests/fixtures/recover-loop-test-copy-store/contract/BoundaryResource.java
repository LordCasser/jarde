public class BoundaryResource extends ContractResource {
    public static final RuntimeException replacement = new RuntimeException("replacement");
    public BoundaryResource(String path) { super(path); }
    public void finish() { ContractResource.events.append("finish;"); }
}
