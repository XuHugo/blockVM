module coin_address::MoveTest{
    //use 0x1::vector;

    // loop test
    // public entry fun loopT(){
    //   let i = 0;
    //       loop {
    //           i = i + 1;
    //       }
      
    // }

    // public entry fun whileT(){
    //   let i = 0;
    //   while(true){
    //       i = i + 1;
    //   }
    // }

    //numerical calculation
    struct Result<T> has key {
        value: T
    }
    
    public entry fun sum_u8(address:&signer, a:u8, b:u8) {
        assert!(a>5,100);
        let c = a + b;
        move_to(address, Result{value: c})
    }

    public entry fun get_result<T: copy+store>(owner: address): T acquires Result {
        borrow_global<Result<T>>(owner).value
    }

    // public entry fun sum_u64(address:&signer, a:u64, b:u64) {
    //     let c = a + b;
    //     move_to(address, Result{value: c});
    // }

    // public entry fun sum_u128(address:&signer, a:u128, b:u128) {
    //     let c = a + b;
    //     move_to(address, Result{value: c});
    // }

    public entry fun sub_u8(address:&signer, a:u8, b:u8) {
        let c = a - b;
        move_to(address, Result{value: c});
    }

    // public entry fun sub_u64(address:&signer, a:u64, b:u64) {
    //     let c = a - b;
    //     move_to(address, Result{value: c});
    // }

    // public entry fun sub_u128(address:&signer, a:u128, b:u128) {
    //     let c = a - b;
    //     move_to(address, Result{value: c});
    // }

    // public entry fun mul_u8(address:&signer, a:u8, b:u8) {
    //     let c = a * b;
    //     move_to(address, Result{value: c});
    // }

    // public entry fun mul_u64(address:&signer, a:u64, b:u64) {
    //     let c = a * b;
    //     move_to(address, Result{value: c});
    // }

    // public entry fun mul_u128(address:&signer, a:u128, b:u128) {
    //     let c = a * b;
    //     move_to(address, Result{value: c});
    // }

    // public entry fun div_u8(address:&signer, a:u8, b:u8) {
    //     let c = a / b;
    //     move_to(address, Result{value: c});
    // }

    // public entry fun div_u64(address:&signer, a:u64, b:u64) {
    //     let c = a / b;
    //     move_to(address, Result{value: c});
    // }

    // public entry fun div_u128(address:&signer, a:u128, b:u128) {
    //     let c = a / b;
    //     move_to(address, Result{value: c});
    // }

    public entry fun rem_u8(address:&signer, a:u8, b:u8) {
        let c = a % b;
        move_to(address, Result{value: c});
    }

    // public entry fun rem_u64(address:&signer, a:u64, b:u64) {
    //     let c = a % b;
    //     move_to(address, Result{value: c});
    // }

    // public entry fun rem_u128(address:&signer, a:u128, b:u128) {
    //     let c = a % b;
    //     move_to(address, Result{value: c});
    // }

    // public entry fun get_result<T: key+copy+store>(owner: address): T acquires Result {
    //     borrow_global<Result<T>>(owner).value
    // }

    // // generic test
    // struct Box<T> has store, key{
    //     value: T
    // }

    // public entry fun create_box<T:store+key>(address: &signer, value: T) {
    //     move_to(address, Box { value });
    // }

    // public entry fun get_box<T: key+copy+store+drop>(owner: address, value: T): bool acquires Box{
    //     let v = borrow_global<Box<T>>(owner).value;
    //     if(&v == &value){
    //         return true
    //     }else{
    //         abort 111
    //     }
    // }

    // public fun get_value<T: key+copy+store>(owner: address): T acquires Box{
    //     borrow_global<Box<T>>(owner).value
    // }

    // // constant test
    const MY_ADDRESS: address = @0x42;
    const ADDRESS_ERROR: u64 = 0;

    public entry fun permissioned(s: &signer) {
        assert!(std::signer::address_of(s) == MY_ADDRESS, 0);
    }

    // // vector test 
    // struct List has store, key{
    //     value: vector<u64>
    // }

    // public entry fun create_vector(address: &signer, num: u64){
    //     let l = vector::empty<u64>();
    //     let i: u64 = 0;
    //     while(i < num){
    //         vector::push_back(&mut l, i);
    //         i = i + 1;
    //     };
    //     move_to(address, List { value: l });
    // }

    // public entry fun get_v(owner: address, index: u64): u64 acquires List{
    //     let list = borrow_global<List>(owner).value;
    //     let a:& u64 = vector::borrow(&list, index);
    //     return *a
    // }

    // // struct test
    // struct User has store,copy,drop{
    //     id: u64,
    //     age: u64
    // }

    // struct Addr has store,key{
    //     value: User
    // }

    // public entry fun new_user(address: &signer, id:u64, age:u64){
    //     move_to(address, Addr{value: User{id, age}});
    // }

    // public entry fun get_user_id(owner: address): u64 acquires Addr{
    //     let v = borrow_global<Addr>(owner).value;
    //     return v.age
    // }

}