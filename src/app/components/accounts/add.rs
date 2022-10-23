use super::super::super::userdata::UserData;
use super::structs::Account;
use cursive::Cursive;

pub fn add_account(s: &mut Cursive, index: Option<u32>, prefix: &str) {
    let data = &mut s.user_data::<UserData>().unwrap();
    let wallet = &mut data.wallets[data.wallet_idx];
    let mut i = 0;
    let mut last = wallet.indexes[i];
    if let Some(index) = index {
        if index < last {
            wallet.indexes.insert(0, index);
            wallet
                .accounts
                .insert(0, Account::with_index(wallet, index, prefix));
            return;
        }
    } else if last != 0 {
        wallet.indexes.insert(0, 0);
        wallet
            .accounts
            .insert(0, Account::with_index(wallet, 0, prefix));
        return;
    }
    for idx in wallet.indexes[1..].iter() {
        if *idx != last + 1 {
            break;
        }
        i += 1;
        last = wallet.indexes[i]
    }

    if let Some(index) = index {
        wallet.indexes.push(index);
        wallet
            .accounts
            .push(Account::with_index(wallet, index, prefix));
    } else {
        wallet.indexes.insert(i + 1, last + 1);
        wallet
            .accounts
            .insert(i + 1, Account::with_index(wallet, last + 1, prefix));
    }
}
