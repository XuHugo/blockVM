module 0xf6b02a2d47b84e845b7e3623355f041bcb36daf1::Collections {
    use 0x1::signer;
    use 0x1::vector;

    // struct Box<T:key + store> has store, key{
    //     value: T
    // }

    // public entry fun create_box<T:key + store>(address: &signer, value: T) {
    //     move_to(address, Box { value });
    // }
    use 0xf6b02a2d47b84e845b7e3623355f041bcb36daf1::Collection;

    struct Result<T> has key {
        value: T
    }
    
    public entry fun sum_u8(address:&signer, a:u8, b:u8) {
        let c = a + b;
        Collection::exists_at(@0x99);
        move_to(address, Result{value: c})
    }
    
}