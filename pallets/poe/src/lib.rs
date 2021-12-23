#![cfg_attr(not(feature = "std"), no_std)]  // if not std,then must bo no std

/// A module for proof of existence

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
   use frame_support::{
       dispatch::DispatchResultWithPostInfo, //可调用函数的返回结果
       pallet_prelude::*}; //runtime开发需要的常用宏
    use frame_system::pallet_prelude::*; // 依赖的一些数据和类型信息
    use sp_std::vec::Vec;
    /// Configure the pallet by specifying the parameters and types on which it depends.

    #[pallet::config] // 定义模块配置接口，使用宏来标记
    pub trait Config: frame_system::Config {
        // 继承系统config接口，只有一个关联类型event
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        // 它可以从本模块的event类型进行转换，并且它的event类型是一个系统event类型

    }

    // 定义pallet结构体承载功能模块
    #[pallet::pallet]
    // 用以表示当前模块依赖的一些存储单元，通过Sore的借口
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::generate_storage_info]
    pub struct Pallet<T>(_);

    //存储单元为：proofs，用来存储存证
    #[pallet::storage]
    // 定义可选的get函数 proofs
    #[pallet::getter(fn proofs)]
    pub(super) type Proofs<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        // 来表示存证的hash值，是key
        Vec<u8>,
        // key所对应的值，是元组
        (T::AccountId, T::BlockNumber),
        //ValueQuery
    >;

    // Pallets use events to inform users when important changes are made.
// Event documentation should end with an array that provides descriptive names for parameters.
// https://docs.substrate.io/v3/runtime/events-and-errors
    #[pallet::event]
    //将accountid转换成前端可识别的类型
    #[pallet::metadata(T::AccountId = "AccountId")]
    // 方便的进行event触发
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    // 用枚举包含event信息
    pub enum Event<T: Config> {
        /// Event emitted when a proof has been claimed. [who, claim]
        ClaimCreated(T::AccountId, Vec<u8>),
        /// Event emitted when a claim is revoked by the owner. [who, claim]
        ClaimRevoked(T::AccountId, Vec<u8>),
    }

    #[pallet::error]
    pub enum Error<T> {
        ProofAlreadyExisted,
        ClaimNotaExist,
        NotClaimOwner,

        // /// The proof has already been claimed.
        // ProofAlreadyClaimed,
        // /// The proof does not exist, so it cannot be revoked.
        // NoSuchProof,
        // /// The proof is claimed by another account, so caller can't revoke it.
        // NotProofOwner,
    }

    // 模块的特殊函数，可以在区块的特殊时期执行
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    //定义可调用函数
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // 创建存证
        #[pallet::weight(0)]
        pub fn create_claim(
            origin: OriginFor<T>,  // 发送方
            claim: Vec<u8>, // 哈希值
        ) -> DispatchResultWithPostInfo { //是result的别名，并且包含了weight的信息
            // 校验是否签名，并得到发送方的id
            let sender = ensure_signed(origin)?;
            //校验存证里面是否已经存在
            ensure!(!Proofs::<T>::contains_key(&claim), Error::<T>::ProofAlreadyExisted);
            // 获取当前的区块
            // Get the block number from the FRAME System pallet.
            let current_block = <frame_system::Pallet<T>>::block_number();
            // 校验完成后执行存储
            // Store the proof with the sender and block number.
            Proofs::<T>::insert(
                &claim, (sender.clone(), current_block));

            // 触发存储事件
            // Emit an event that the claim was created.
            Self::deposit_event(Event::ClaimCreated(sender, claim));
            // 定义了触发事件
            Ok(().into())
        }

        // 吊销存证
        #[pallet::weight(0)]
        pub fn revoke_claim(
            origin: OriginFor<T>,
            claim: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://docs.substrate.io/v3/runtime/origins
            let sender = ensure_signed(origin)?;

            // // Verify that the specified proof has been claimed.
            // ensure!(Proofs::<T>::contains_key(&proof), Error::<T>::NoSuchProof);

            // Get owner of the claim.
            // 校验当前存储是否存在这个值
            let (owner, _) = Proofs::<T>::get(&claim).
                ok_or(Error::<T>::ClaimNotaExist)?;
            // ok_or : a function in option
            // 如果get的结果是None，则把None转换为ok_or的后面函数的信息
            // 如果有值，则用？把值取出来

            // 校验当前交易的发送方是不是proof的所有人
            // Verify that sender of the current call is the claim owner.
            ensure!(sender == owner, Error::<T>::NotClaimOwner);

            // Remove claim from storage.
            Proofs::<T>::remove(&claim);

            // 删除存证后触发一个事件
            // Emit an event that the claim was erased.
            Self::deposit_event(Event::ClaimRevoked(sender, claim));

            Ok(().into())

        }
    }
}