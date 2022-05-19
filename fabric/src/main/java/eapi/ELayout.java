package eapi;

import java.util.List;

public class ELayout<T> {

    private final List<T> layout;

    public ELayout(List<T> elements) {
        this.layout = List.copyOf(elements);
    }

    public List<T> getElements() {
        return this.layout;
    }

}
