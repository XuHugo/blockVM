module 0xf6b02a2d47b84e845b7e3623355f041bcb36daf1::IMCoin {
  use Std::Signer;

  const MAX_SUPPLY: u64 = 1024;
  const ISSUER: address = @0xf6b02a2d47b84e845b7e3623355f041bcb36daf1;

  struct Coin has key {
    supply: u64,
    holders: u64,
  }
  struct Balance has key {
    value: u64
  }

  // complete this function
  public fun issue(issuer: &signer) {
    let issuer_addr = Signer::address_of(issuer);
    assert!(issuer_addr == ISSUER,1001);
    assert!(!exists<Coin>(ISSUER), 1002);
    move_to<Coin>(issuer, Coin{supply:0, holders:0});
  }

  // complete this function
  public fun supply(): u64 acquires Coin {
    assert!(exists<Coin>(ISSUER), 1003);
    borrow_global<Coin>(ISSUER).supply
  }

  public fun holders(): u64 acquires Coin {
    assert!(exists<Coin>(ISSUER), 1003);

    borrow_global<Coin>(ISSUER).holders
  }

  // complete this function
  public fun register(account: &signer) acquires Coin{
    assert!(exists<Coin>(ISSUER), 1003);
    assert!(!exists<Balance>(Signer::address_of(account)), 2001);
    
    let v = &mut borrow_global_mut<Coin>(ISSUER).holders;
    *v = *v + 1;
    move_to<Balance>(account, Balance{value:0});
  }

  // complete this function
  public fun deposit(issuer: &signer, receiver: address, amount: u64) acquires Coin,Balance{
    let issuer_addr = Signer::address_of(issuer);
    assert!(issuer_addr == ISSUER, 1001);
    assert!(exists<Coin>(ISSUER), 1003);
    assert!(exists<Balance>(receiver), 2002);
    let v = &mut borrow_global_mut<Coin>(ISSUER).supply;
    assert!(*v + amount <= MAX_SUPPLY, 1004);
    *v =  *v + amount;

    let v = &mut borrow_global_mut<Balance>(receiver).value;
    *v = *v + amount;
  }

  // complete this function
  public fun balance(addr: address): u64 acquires Balance{
    assert!(exists<Balance>(addr), 2002);
    
    borrow_global<Balance>(addr).value
  }

  // complete this function
  public fun transfer(sender: &signer, receiver: address, amount: u64) acquires Balance{
    assert!(Signer::address_of(sender) != receiver,2003);
    sub_balance(Signer::address_of(sender), amount);
    add_balance(receiver, amount);
  }

  fun add_balance(addr: address, amount: u64) acquires Balance {
    assert!(exists<Balance>(addr), 2002);

    let value_ref = &mut borrow_global_mut<Balance>(addr).value;
    *value_ref = *value_ref + amount;
  }

  fun sub_balance(addr: address, amount: u64) acquires Balance {
    assert!(exists<Balance>(addr), 2002);

    let value_ref = &mut borrow_global_mut<Balance>(addr).value;
    assert!(*value_ref >= amount, 2004);
    *value_ref = *value_ref - amount;
  }
}
