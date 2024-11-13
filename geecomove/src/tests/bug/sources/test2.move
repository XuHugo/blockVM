module 0xf6b02a2d47b84e845b7e3623355f041bcb36daf1::Collection {
    use 0x1::signer;
    use 0x1::vector;

    struct Item has store {
        value: u64,
    }
    struct Ii  {
        value: u64,
        values: u64,
        valuess: u64,
    }

    struct RetData  {
        value: u64,
        vec: vector<u8>,
    }

    struct Collections has key {
        items: vector<Item>
    }

    public entry fun start_collection(account: &signer) {
        move_to<Collections>(account, Collections {
            items: vector::empty<Item>()
        })
    }

    public entry fun exists_at(at: address):bool{
        exists<Collections>(at)
    }

    public entry fun size(account: &signer): u64 acquires Collections{
        let owner = signer::address_of(account);
        let collection = borrow_global<Collections>(owner);
        vector::length(&collection.items)
    }

    public entry fun sizes(account: &signer) acquires Collections{
        let owner = signer::address_of(account);
        let collection = borrow_global<Collections>(owner);
        vector::length(&collection.items);
    }

    public entry fun add_item(account: &signer) acquires Collections {
        let collection = borrow_global_mut<Collections>(signer::address_of(account));

        vector::push_back(&mut collection.items, Item {value:6});
    }

    public entry fun destory(account: &signer):vector<Item> acquires Collections {

        let collection = move_from<Collections>(signer::address_of(account));

        let Collections { items } = collection;
        items
    }

    public entry fun destory2(account: &signer):Collections acquires Collections {

        let collection = move_from<Collections>(signer::address_of(account));

        collection
    }

    public entry fun destory_resource(account: &signer) acquires Collections{

        let Collections { items }= destory2(account);

        loop{
            let len = vector::length(&items);
            if (len != 0){
                let Item{value:_} = vector::pop_back(&mut items);
            }else{
                break
            }

        };

        vector::destroy_empty(items);
    }

    public entry fun get(account: &signer):Ii acquires Collections {

        let collection = borrow_global<Collections>(signer::address_of(account));

        let len = vector::length(&collection.items);

        Ii{value: len, values:33, valuess:333}
    }

    public entry fun get2(account_addr : address):RetData acquires Collections {

        let collection = borrow_global<Collections>(account_addr);

        let len = vector::length(&collection.items);
        let vec = vector[4u8, 6u8, 8u8];
        RetData{value: len, vec}
    }

    public entry fun get3():RetData  acquires Collections{

        let collection = borrow_global<Collections>(@0xf6b02a2d47b84e845b7e3623355f041bcb36daf1);

        let len = vector::length(&collection.items);
        //vector::empty<Item>()
        let vec = vector[3u8, 6u8, 9u8];
        RetData{value: len, vec}
    }

    public entry fun loopT(){
        let i = 0;
        loop{
            i = i + 1;
        }
    }

    
}