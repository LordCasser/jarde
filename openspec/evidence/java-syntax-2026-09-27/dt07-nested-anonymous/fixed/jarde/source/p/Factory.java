// jarde: presentation of `p/Factory` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package p;

public interface Factory {
    // jarde: no body: the member `make()Lp/Action;` is declared abstract and its declaration carries no Code attribute
    public abstract p.Action make();
}
