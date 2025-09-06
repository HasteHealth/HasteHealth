pub enum V3ActClass {
    _ActClassRecordOrganizer(Option<Element>),
    DOC(Option<Element>),
    CDALVLONE(Option<Element>),
    CATEGORY(Option<Element>),
    DOCBODY(Option<Element>),
    DOCSECT(Option<Element>),
    TOPIC(Option<Element>),
    EHR(Option<Element>),
    CLUSTER(Option<Element>),
    ACCM(Option<Element>),
    ACCT(Option<Element>),
    ACSN(Option<Element>),
    ADJUD(Option<Element>),
    CACT(Option<Element>),
    CNTRCT(Option<Element>),
    COV(Option<Element>),
    CONC(Option<Element>),
    CONS(Option<Element>),
    CONTREG(Option<Element>),
    CTTEVENT(Option<Element>),
    DISPACT(Option<Element>),
    EXPOS(Option<Element>),
    INC(Option<Element>),
    INFRM(Option<Element>),
    INVE(Option<Element>),
    LIST(Option<Element>),
    MPROT(Option<Element>),
    OBS(Option<Element>),
    ROIBND(Option<Element>),
    ROIOVL(Option<Element>),
    _SubjectBodyPosition(Option<Element>),
    RTRD(Option<Element>),
    TRD(Option<Element>),
    CASE(Option<Element>),
    DETPOL(Option<Element>),
    EXP(Option<Element>),
    LOC(Option<Element>),
    PHN(Option<Element>),
    POL(Option<Element>),
    SEQ(Option<Element>),
    SEQVAR(Option<Element>),
    OBSCOR(Option<Element>),
    POSACC(Option<Element>),
    POSCOORD(Option<Element>),
    PCPR(Option<Element>),
    POLICY(Option<Element>),
    PROC(Option<Element>),
    SPECCOLLECT(Option<Element>),
    REG(Option<Element>),
    REV(Option<Element>),
    SPCTRT(Option<Element>),
    SPLY(Option<Element>),
    STORE(Option<Element>),
    SUBST(Option<Element>),
    TRFR(Option<Element>),
    TRNS(Option<Element>),
    XACT(Option<Element>),
    _ActClassContainer(Option<Element>),
}
pub enum V3ParticipationIndirectTarget {
    _ParticipationAncillary(Option<Element>),
    _ParticipationInformationGenerator(Option<Element>),
    ENT(Option<Element>),
    CST(Option<Element>),
    DIR(Option<Element>),
    TPA(Option<Element>),
    NRD(Option<Element>),
    RDV(Option<Element>),
    EXPTRGT(Option<Element>),
    EXSRC(Option<Element>),
    SPC(Option<Element>),
    IND(Option<Element>),
    IRCP(Option<Element>),
    LOC(Option<Element>),
    PRF(Option<Element>),
    RESP(Option<Element>),
    VRF(Option<Element>),
}
pub enum ContractAssetcontext {}
pub enum StrandType {}
pub enum V3RoleClassRoot {
    _RoleClassAssociative(Option<Element>),
    _RoleClassRelationshipFormal(Option<Element>),
    ASSIGNED(Option<Element>),
    CON(Option<Element>),
    GUARD(Option<Element>),
    CLAIM(Option<Element>),
    NAMED(Option<Element>),
    PROG(Option<Element>),
    MIL(Option<Element>),
    CASEBJ(Option<Element>),
    RESBJ(Option<Element>),
    NOT(Option<Element>),
    PROV(Option<Element>),
    CAREGIVER(Option<Element>),
    PRS(Option<Element>),
    SELF(Option<Element>),
    ACCESS(Option<Element>),
    ADJY(Option<Element>),
    BOND(Option<Element>),
    CONY(Option<Element>),
    ADMM(Option<Element>),
    BIRTHPL(Option<Element>),
    DEATHPLC(Option<Element>),
    DST(Option<Element>),
    EXLOC(Option<Element>),
    DSDLOC(Option<Element>),
    ISDLOC(Option<Element>),
    EXPR(Option<Element>),
    HLD(Option<Element>),
    HLTHCHRT(Option<Element>),
    IDENT(Option<Element>),
    MANU(Option<Element>),
    MNT(Option<Element>),
    OWN(Option<Element>),
    RGPR(Option<Element>),
    TERR(Option<Element>),
    USED(Option<Element>),
    WRTE(Option<Element>),
    _RoleClassOntological(Option<Element>),
    SAME(Option<Element>),
    SUBY(Option<Element>),
    GRIC(Option<Element>),
    _RoleClassPartitive(Option<Element>),
    EXPVECTOR(Option<Element>),
    FOMITE(Option<Element>),
    ACTI(Option<Element>),
    ADJV(Option<Element>),
    ADTV(Option<Element>),
    BASE(Option<Element>),
    CNTM(Option<Element>),
    IACT(Option<Element>),
    MECH(Option<Element>),
    STOR(Option<Element>),
    ACTM(Option<Element>),
    ALQT(Option<Element>),
    ISLT(Option<Element>),
}
pub enum QuestionnaireCategory {}
pub enum V3TriggerEventID {}
pub enum V3ActUncertainty {}
pub enum V3EntityNameUse {
    ABC(Option<Element>),
    IDE(Option<Element>),
    SYL(Option<Element>),
    OR(Option<Element>),
    A(Option<Element>),
    PHON(Option<Element>),
    SNDX(Option<Element>),
}
pub enum ActionGroupingBehavior {}
pub enum V3ObservationCategory {}
pub enum ClaimType {}
pub enum BenefitTerm {}
pub enum ProvenanceHistoryAgentType {}
pub enum ContractDataMeaning {}
pub enum MapSourceListMode {}
pub enum CodesystemContentMode {}
pub enum AssertResponseCodeTypes {}
pub enum ActionType {}
pub enum ReferencerangeMeaning {
    Normal(Option<Element>),
    Recommended(Option<Element>),
    Treatment(Option<Element>),
    Therapeutic(Option<Element>),
    PrePuberty(Option<Element>),
    Follicular(Option<Element>),
    Midcycle(Option<Element>),
    Luteal(Option<Element>),
    Postmenopausal(Option<Element>),
}
pub enum SupplydeliveryType {}
pub enum V3Hl7Realm {
    AffiliateRealms(Option<Element>),
    C1(Option<Element>),
    GB(Option<Element>),
    R1(Option<Element>),
    X1(Option<Element>),
    ZZ(Option<Element>),
}
pub enum V3PatientImportance {}
pub enum ConditionSeverity {}
pub enum ReportParticipantType {}
pub enum V3ObservationMethod {
    ALGM(Option<Element>),
    GINT(Option<Element>),
    PCR(Option<Element>),
    AVERAGE(Option<Element>),
    COUNT(Option<Element>),
    MAX(Option<Element>),
    MEDIAN(Option<Element>),
    MIN(Option<Element>),
    MODE(Option<Element>),
    STDEVP(Option<Element>),
    STDEVS(Option<Element>),
    SUM(Option<Element>),
    VARIANCEP(Option<Element>),
    VARIANCES(Option<Element>),
    VDOC(Option<Element>),
    VREG(Option<Element>),
    VTOKEN(Option<Element>),
    VVOICE(Option<Element>),
    V0240(Option<Element>),
    V0241(Option<Element>),
    V0242(Option<Element>),
    V0272(Option<Element>),
    V0273(Option<Element>),
    V0274(Option<Element>),
    V0275(Option<Element>),
    V0275a(Option<Element>),
    V0276(Option<Element>),
    V0277(Option<Element>),
    V0278(Option<Element>),
    V0279(Option<Element>),
}
pub enum NameAssemblyOrder {}
pub enum MedicationStatementCategory {}
pub enum ContractLegalstate {}
pub enum EventCapabilityMode {}
pub enum CopyNumberEvent {}
pub enum Participationstatus {}
pub enum GraphCompartmentUse {}
pub enum V3EntityClassDevice {
    HCE(Option<Element>),
    LIV(Option<Element>),
    ANM(Option<Element>),
    MIC(Option<Element>),
    PLNT(Option<Element>),
    MAT(Option<Element>),
    CONT(Option<Element>),
    DEV(Option<Element>),
    ORG(Option<Element>),
    NAT(Option<Element>),
    PLC(Option<Element>),
    RGRP(Option<Element>),
}
pub enum NameUse {
    Maiden(Option<Element>),
}
pub enum V3GenderStatus {}
pub enum Slotstatus {}
pub enum EntformulaAdditive {}
pub enum RequestStatus {}
pub enum MeasureReportType {}
pub enum ContractAssetscope {}
pub enum HttpOperations {}
pub enum DefinedTypes {}
pub enum ServiceProduct {}
pub enum SeriesPerformerFunction {}
pub enum SpecialValues {}
pub enum OrganizationType {}
pub enum AuditEventSubType {
    HistoryInstance(Option<Element>),
    HistoryType(Option<Element>),
    HistorySystem(Option<Element>),
    SearchType(Option<Element>),
    SearchSystem(Option<Element>),
}
pub enum SpecimenStatus {}
pub enum ObjectRole {}
pub enum ContractContentDerivative {}
pub enum ExOnsettype {}
pub enum Udi {}
pub enum EpisodeOfCareStatus {}
pub enum Additionalmaterials {}
pub enum V3ParticipationType {
    _ParticipationAncillary(Option<Element>),
    _ParticipationInformationGenerator(Option<Element>),
    ENT(Option<Element>),
    CST(Option<Element>),
    DIR(Option<Element>),
    TPA(Option<Element>),
    NRD(Option<Element>),
    RDV(Option<Element>),
    EXPTRGT(Option<Element>),
    EXSRC(Option<Element>),
    SPC(Option<Element>),
    IND(Option<Element>),
    IRCP(Option<Element>),
    LOC(Option<Element>),
    PRF(Option<Element>),
    RESP(Option<Element>),
    VRF(Option<Element>),
}
pub enum AuditEventType {}
pub enum V3QueryPriority {}
pub enum LocationMode {}
pub enum NhinPurposeofuse {}
pub enum V3EncounterAdmissionSource {}
pub enum BundleType {}
pub enum PermittedDataType {}
pub enum ExtensionContextType {}
pub enum ResearchSubjectStatus {}
pub enum MapModelMode {}
pub enum OrganizationRole {}
pub enum TransactionMode {}
pub enum SearchXpathUsage {}
pub enum WrittenLanguage {}
pub enum V3RoleLinkStatus {
    ACTIVE(Option<Element>),
    CANCELLED(Option<Element>),
    COMPLETED(Option<Element>),
    PENDING(Option<Element>),
}
pub enum OperationParameterUse {}
pub enum V3ActRelationshipCheckpoint {}
pub enum PostalAddressUse {}
pub enum ResearchStudyReasonStopped {}
pub enum MetricCalibrationType {}
pub enum MeasureScoring {}
pub enum MedicationdispenseStatus {}
pub enum SpecimenContainedPreference {}
pub enum V3EntityNamePartQualifierR2 {
    SP(Option<Element>),
    CON(Option<Element>),
    DEV(Option<Element>),
    FLAV(Option<Element>),
    FORMUL(Option<Element>),
    FRM(Option<Element>),
    INV(Option<Element>),
    POPUL(Option<Element>),
    SCI(Option<Element>),
    STR(Option<Element>),
    TIME(Option<Element>),
    TMK(Option<Element>),
    USE(Option<Element>),
    AC(Option<Element>),
    HON(Option<Element>),
    NB(Option<Element>),
    PR(Option<Element>),
}
pub enum V3Confidentiality {
    L(Option<Element>),
    M(Option<Element>),
    N(Option<Element>),
    R(Option<Element>),
    U(Option<Element>),
    V(Option<Element>),
    B(Option<Element>),
    D(Option<Element>),
    I(Option<Element>),
    ETH(Option<Element>),
    HIV(Option<Element>),
    PSY(Option<Element>),
    SDV(Option<Element>),
    C(Option<Element>),
    S(Option<Element>),
    T(Option<Element>),
}
pub enum V3HL7UpdateMode {
    ESA(Option<Element>),
    ESAC(Option<Element>),
    ESC(Option<Element>),
    ESD(Option<Element>),
}
pub enum NarrativeStatus {}
pub enum V3AdministrativeGender {}
pub enum ReportResultCodes {}
pub enum ExBenefitcategory {}
pub enum ProvenanceAgentType {
    Legal(Option<Element>),
}
pub enum RestfulSecurityService {}
pub enum CodeSearchSupport {}
pub enum CoverageFinancialException {}
pub enum PlanDefinitionType {}
pub enum TestscriptOperationCodes {}
pub enum RiskEstimateType {}
pub enum ImmunizationRecommendationReason {}
pub enum ConditionClinical {
    Recurrence(Option<Element>),
    Relapse(Option<Element>),
    Remission(Option<Element>),
    Resolved(Option<Element>),
}
pub enum ActionRequiredBehavior {}
pub enum EncounterLocationStatus {}
pub enum ContractType {}
pub enum LinkageType {}
pub enum V3Ethnicity {
    V21378(Option<Element>),
    V21485(Option<Element>),
    V21550(Option<Element>),
    V21659(Option<Element>),
    V21782(Option<Element>),
    V21808(Option<Element>),
    V21824(Option<Element>),
    V21840(Option<Element>),
}
pub enum SubstanceCategory {}
pub enum EventOrRequestResourceTypes {}
pub enum MedicationAdminCategory {}
pub enum ResourceTypeLink {}
pub enum ObjectLifecycleEvents {}
pub enum V3Hl7PublishingSection {}
pub enum V3ActClassROI {
    _ActClassRecordOrganizer(Option<Element>),
    DOC(Option<Element>),
    CDALVLONE(Option<Element>),
    CATEGORY(Option<Element>),
    DOCBODY(Option<Element>),
    DOCSECT(Option<Element>),
    TOPIC(Option<Element>),
    EHR(Option<Element>),
    CLUSTER(Option<Element>),
    ACCM(Option<Element>),
    ACCT(Option<Element>),
    ACSN(Option<Element>),
    ADJUD(Option<Element>),
    CACT(Option<Element>),
    CNTRCT(Option<Element>),
    COV(Option<Element>),
    CONC(Option<Element>),
    CONS(Option<Element>),
    CONTREG(Option<Element>),
    CTTEVENT(Option<Element>),
    DISPACT(Option<Element>),
    EXPOS(Option<Element>),
    INC(Option<Element>),
    INFRM(Option<Element>),
    INVE(Option<Element>),
    LIST(Option<Element>),
    MPROT(Option<Element>),
    OBS(Option<Element>),
    ROIBND(Option<Element>),
    ROIOVL(Option<Element>),
    _SubjectBodyPosition(Option<Element>),
    RTRD(Option<Element>),
    TRD(Option<Element>),
    CASE(Option<Element>),
    DETPOL(Option<Element>),
    EXP(Option<Element>),
    LOC(Option<Element>),
    PHN(Option<Element>),
    POL(Option<Element>),
    SEQ(Option<Element>),
    SEQVAR(Option<Element>),
    OBSCOR(Option<Element>),
    POSACC(Option<Element>),
    POSCOORD(Option<Element>),
    PCPR(Option<Element>),
    POLICY(Option<Element>),
    PROC(Option<Element>),
    SPECCOLLECT(Option<Element>),
    REG(Option<Element>),
    REV(Option<Element>),
    SPCTRT(Option<Element>),
    SPLY(Option<Element>),
    STORE(Option<Element>),
    SUBST(Option<Element>),
    TRFR(Option<Element>),
    TRNS(Option<Element>),
    XACT(Option<Element>),
    _ActClassContainer(Option<Element>),
}
pub enum V3EntityClassPlace {
    HCE(Option<Element>),
    LIV(Option<Element>),
    ANM(Option<Element>),
    MIC(Option<Element>),
    PLNT(Option<Element>),
    MAT(Option<Element>),
    CONT(Option<Element>),
    DEV(Option<Element>),
    ORG(Option<Element>),
    NAT(Option<Element>),
    PLC(Option<Element>),
    RGRP(Option<Element>),
}
pub enum V3HL7ContextConductionStyle {}
pub enum MedicationdispenseStatusReason {}
pub enum BindingStrength {}
pub enum DefinitionStatus {}
pub enum V3ParticipationFunction {
    _AuthorizedReceiverParticipationFunction(Option<Element>),
    _ConsenterParticipationFunction(Option<Element>),
    _OverriderParticipationFunction(Option<Element>),
    _PayorParticipationFunction(Option<Element>),
    _SponsorParticipationFunction(Option<Element>),
    _UnderwriterParticipationFunction(Option<Element>),
}
pub enum ConditionalReadStatus {}
pub enum V3IdentifierScope {}
pub enum NamingsystemIdentifierType {}
pub enum V3PolicyHolderRole {}
pub enum V3Hl7PublishingDomain {}
pub enum V3EntityNamePartQualifier {
    AC(Option<Element>),
    AD(Option<Element>),
    BR(Option<Element>),
    CL(Option<Element>),
    IN(Option<Element>),
    LS(Option<Element>),
    NB(Option<Element>),
    PR(Option<Element>),
    SP(Option<Element>),
    TITLE(Option<Element>),
    VV(Option<Element>),
    CON(Option<Element>),
    DEV(Option<Element>),
    FLAV(Option<Element>),
    FORMUL(Option<Element>),
    FRM(Option<Element>),
    INV(Option<Element>),
    POPUL(Option<Element>),
    SCI(Option<Element>),
    STR(Option<Element>),
    TIME(Option<Element>),
    TMK(Option<Element>),
    USE(Option<Element>),
    _PersonNamePartAffixTypes(Option<Element>),
    _PersonNamePartChangeQualifier(Option<Element>),
    _PersonNamePartMiscQualifier(Option<Element>),
}
pub enum IdentifierUse {}
pub enum CoverageeligibilityresponseExAuthSupport {}
pub enum EventTiming {}
pub enum ImagingstudyStatus {}
pub enum AnimalGenderstatus {}
pub enum DiagnosisRole {}
pub enum ContractAction {}
pub enum V3ActRelationshipConditional {
    _ActClassTemporallyPertains(Option<Element>),
    _ActRelationshipAccounting(Option<Element>),
    CHRG(Option<Element>),
    COST(Option<Element>),
    CREDIT(Option<Element>),
    DEBIT(Option<Element>),
    _ActRelationshipConditional(Option<Element>),
    BLOCK(Option<Element>),
    DIAG(Option<Element>),
    IMM(Option<Element>),
    MITGT(Option<Element>),
    PRYLX(Option<Element>),
    TREAT(Option<Element>),
    _ActRelationshipTemporallyPertains(Option<Element>),
    ENE(Option<Element>),
    CONCURRENT(Option<Element>),
    SBSECWE(Option<Element>),
    ENS(Option<Element>),
    SNE(Option<Element>),
    SNS(Option<Element>),
    SCWSEBE(Option<Element>),
    SCWSEAE(Option<Element>),
    EAE(Option<Element>),
    SBEEAE(Option<Element>),
    SAS(Option<Element>),
    EAEORECW(Option<Element>),
    OVERLAP(Option<Element>),
    SBSEASEBE(Option<Element>),
    SBE(Option<Element>),
    SBSEBE(Option<Element>),
    EBS(Option<Element>),
    SBS(Option<Element>),
    AUTH(Option<Element>),
    CAUS(Option<Element>),
    COMP(Option<Element>),
    STEP(Option<Element>),
    COVBY(Option<Element>),
    DRIV(Option<Element>),
    ELNK(Option<Element>),
    EVID(Option<Element>),
    EXACBY(Option<Element>),
    EXPL(Option<Element>),
    INTF(Option<Element>),
    ITEMSLOC(Option<Element>),
    LIMIT(Option<Element>),
    META(Option<Element>),
    MFST(Option<Element>),
    NAME(Option<Element>),
    OUTC(Option<Element>),
    OBJC(Option<Element>),
    OBJF(Option<Element>),
    PERT(Option<Element>),
    PREV(Option<Element>),
    REFR(Option<Element>),
    REFV(Option<Element>),
    RELVBY(Option<Element>),
    SEQL(Option<Element>),
    OCCR(Option<Element>),
    OREF(Option<Element>),
    SCH(Option<Element>),
    VRXCRPT(Option<Element>),
    SPRT(Option<Element>),
    SUBJ(Option<Element>),
    SUMM(Option<Element>),
    VALUE(Option<Element>),
}
pub enum CommunicationCategory {}
pub enum DesignationUse {}
pub enum Inactive {
    _ActMoodPotential(Option<Element>),
    EVN(Option<Element>),
    INT(Option<Element>),
    _ActMoodActRequest(Option<Element>),
    PRP(Option<Element>),
    APT(Option<Element>),
    CRT(Option<Element>),
    PRMSCRT(Option<Element>),
    RQOCRT(Option<Element>),
    EXPEC(Option<Element>),
    OPT(Option<Element>),
}
pub enum CodesystemAltcodeKind {}
pub enum ConsentStateCodes {}
pub enum ResearchStudyPrimPurpType {}
pub enum DeviceDefinitionStatus {}
pub enum VerificationresultNeed {}
pub enum V3ActClassObservation {
    _ActClassRecordOrganizer(Option<Element>),
    DOC(Option<Element>),
    CDALVLONE(Option<Element>),
    CATEGORY(Option<Element>),
    DOCBODY(Option<Element>),
    DOCSECT(Option<Element>),
    TOPIC(Option<Element>),
    EHR(Option<Element>),
    CLUSTER(Option<Element>),
    ACCM(Option<Element>),
    ACCT(Option<Element>),
    ACSN(Option<Element>),
    ADJUD(Option<Element>),
    CACT(Option<Element>),
    CNTRCT(Option<Element>),
    COV(Option<Element>),
    CONC(Option<Element>),
    CONS(Option<Element>),
    CONTREG(Option<Element>),
    CTTEVENT(Option<Element>),
    DISPACT(Option<Element>),
    EXPOS(Option<Element>),
    INC(Option<Element>),
    INFRM(Option<Element>),
    INVE(Option<Element>),
    LIST(Option<Element>),
    MPROT(Option<Element>),
    OBS(Option<Element>),
    ROIBND(Option<Element>),
    ROIOVL(Option<Element>),
    _SubjectBodyPosition(Option<Element>),
    RTRD(Option<Element>),
    TRD(Option<Element>),
    CASE(Option<Element>),
    DETPOL(Option<Element>),
    EXP(Option<Element>),
    LOC(Option<Element>),
    PHN(Option<Element>),
    POL(Option<Element>),
    SEQ(Option<Element>),
    SEQVAR(Option<Element>),
    OBSCOR(Option<Element>),
    POSACC(Option<Element>),
    POSCOORD(Option<Element>),
    PCPR(Option<Element>),
    POLICY(Option<Element>),
    PROC(Option<Element>),
    SPECCOLLECT(Option<Element>),
    REG(Option<Element>),
    REV(Option<Element>),
    SPCTRT(Option<Element>),
    SPLY(Option<Element>),
    STORE(Option<Element>),
    SUBST(Option<Element>),
    TRFR(Option<Element>),
    TRNS(Option<Element>),
    XACT(Option<Element>),
    _ActClassContainer(Option<Element>),
}
pub enum V3ContainerCap {
    CHILD(Option<Element>),
    EASY(Option<Element>),
}
pub enum ActionConditionKind {}
pub enum C80DocTypecodes {}
pub enum QuestionnaireresponseMode {}
pub enum V3ModifyIndicator {}
pub enum CodesystemHierarchyMeaning {}
pub enum V3EquipmentAlertLevel {}
pub enum V3AcknowledgementType {}
pub enum V3ActUSPrivacyLaw {
    V42CFRPart2(Option<Element>),
    CommonRule(Option<Element>),
    HIPAANOPP(Option<Element>),
    HIPAAPsyNotes(Option<Element>),
    HIPAASelfPay(Option<Element>),
    Title38Section7332(Option<Element>),
}
pub enum ReferenceHandlingPolicy {}
pub enum V3Hl7ApprovalStatus {}
pub enum V3ActInvoiceElementModifier {}
pub enum QualityType {}
pub enum AllergIntolSubstanceExpRisk {}
pub enum ContainerMaterial {}
pub enum V3SubstitutionCondition {
    CONFIRM(Option<Element>),
    NOTIFY(Option<Element>),
}
pub enum VisionBaseCodes {}
pub enum Tooth {}
pub enum V3ActPriority {
    CSP(Option<Element>),
    CSR(Option<Element>),
}
pub enum ClinicalimpressionStatus {}
pub enum V3VaccineManufacturer {}
pub enum V3ActRelationshipType {
    _ActClassTemporallyPertains(Option<Element>),
    _ActRelationshipAccounting(Option<Element>),
    CHRG(Option<Element>),
    COST(Option<Element>),
    CREDIT(Option<Element>),
    DEBIT(Option<Element>),
    _ActRelationshipConditional(Option<Element>),
    BLOCK(Option<Element>),
    DIAG(Option<Element>),
    IMM(Option<Element>),
    MITGT(Option<Element>),
    PRYLX(Option<Element>),
    TREAT(Option<Element>),
    _ActRelationshipTemporallyPertains(Option<Element>),
    ENE(Option<Element>),
    CONCURRENT(Option<Element>),
    SBSECWE(Option<Element>),
    ENS(Option<Element>),
    SNE(Option<Element>),
    SNS(Option<Element>),
    SCWSEBE(Option<Element>),
    SCWSEAE(Option<Element>),
    EAE(Option<Element>),
    SBEEAE(Option<Element>),
    SAS(Option<Element>),
    EAEORECW(Option<Element>),
    OVERLAP(Option<Element>),
    SBSEASEBE(Option<Element>),
    SBE(Option<Element>),
    SBSEBE(Option<Element>),
    EBS(Option<Element>),
    SBS(Option<Element>),
    AUTH(Option<Element>),
    CAUS(Option<Element>),
    COMP(Option<Element>),
    STEP(Option<Element>),
    COVBY(Option<Element>),
    DRIV(Option<Element>),
    ELNK(Option<Element>),
    EVID(Option<Element>),
    EXACBY(Option<Element>),
    EXPL(Option<Element>),
    INTF(Option<Element>),
    ITEMSLOC(Option<Element>),
    LIMIT(Option<Element>),
    META(Option<Element>),
    MFST(Option<Element>),
    NAME(Option<Element>),
    OUTC(Option<Element>),
    OBJC(Option<Element>),
    OBJF(Option<Element>),
    PERT(Option<Element>),
    PREV(Option<Element>),
    REFR(Option<Element>),
    REFV(Option<Element>),
    RELVBY(Option<Element>),
    SEQL(Option<Element>),
    OCCR(Option<Element>),
    OREF(Option<Element>),
    SCH(Option<Element>),
    VRXCRPT(Option<Element>),
    SPRT(Option<Element>),
    SUBJ(Option<Element>),
    SUMM(Option<Element>),
    VALUE(Option<Element>),
}
pub enum V3ActRelationshipFulfills {
    _ActClassTemporallyPertains(Option<Element>),
    _ActRelationshipAccounting(Option<Element>),
    CHRG(Option<Element>),
    COST(Option<Element>),
    CREDIT(Option<Element>),
    DEBIT(Option<Element>),
    _ActRelationshipConditional(Option<Element>),
    BLOCK(Option<Element>),
    DIAG(Option<Element>),
    IMM(Option<Element>),
    MITGT(Option<Element>),
    PRYLX(Option<Element>),
    TREAT(Option<Element>),
    _ActRelationshipTemporallyPertains(Option<Element>),
    ENE(Option<Element>),
    CONCURRENT(Option<Element>),
    SBSECWE(Option<Element>),
    ENS(Option<Element>),
    SNE(Option<Element>),
    SNS(Option<Element>),
    SCWSEBE(Option<Element>),
    SCWSEAE(Option<Element>),
    EAE(Option<Element>),
    SBEEAE(Option<Element>),
    SAS(Option<Element>),
    EAEORECW(Option<Element>),
    OVERLAP(Option<Element>),
    SBSEASEBE(Option<Element>),
    SBE(Option<Element>),
    SBSEBE(Option<Element>),
    EBS(Option<Element>),
    SBS(Option<Element>),
    AUTH(Option<Element>),
    CAUS(Option<Element>),
    COMP(Option<Element>),
    STEP(Option<Element>),
    COVBY(Option<Element>),
    DRIV(Option<Element>),
    ELNK(Option<Element>),
    EVID(Option<Element>),
    EXACBY(Option<Element>),
    EXPL(Option<Element>),
    INTF(Option<Element>),
    ITEMSLOC(Option<Element>),
    LIMIT(Option<Element>),
    META(Option<Element>),
    MFST(Option<Element>),
    NAME(Option<Element>),
    OUTC(Option<Element>),
    OBJC(Option<Element>),
    OBJF(Option<Element>),
    PERT(Option<Element>),
    PREV(Option<Element>),
    REFR(Option<Element>),
    REFV(Option<Element>),
    RELVBY(Option<Element>),
    SEQL(Option<Element>),
    OCCR(Option<Element>),
    OREF(Option<Element>),
    SCH(Option<Element>),
    VRXCRPT(Option<Element>),
    SPRT(Option<Element>),
    SUBJ(Option<Element>),
    SUMM(Option<Element>),
    VALUE(Option<Element>),
}
pub enum AssertDirectionCodes {}
pub enum ReportActionResultCodes {}
pub enum V3RelationalOperator {}
pub enum FmItemtype {}
pub enum RestfulCapabilityMode {}
pub enum UnitsOfTime {}
pub enum ImplantStatus {}
pub enum V3DocumentStorage {
    AA(Option<Element>),
}
pub enum V3AddressUse {
    BAD(Option<Element>),
    CONF(Option<Element>),
    H(Option<Element>),
    OLD(Option<Element>),
    TMP(Option<Element>),
    WP(Option<Element>),
    PHYS(Option<Element>),
    PST(Option<Element>),
    AS(Option<Element>),
    EC(Option<Element>),
    MC(Option<Element>),
    PG(Option<Element>),
}
pub enum BodysiteLaterality {}
pub enum ExDiagnosisrelatedgroup {}
pub enum RelationType {}
pub enum ContractDefinitionType {}
pub enum RecommendationStrength {}
pub enum PaymentStatus {}
pub enum V3ProvenanceEventCurrentStateDC {}
pub enum ContractTermType {}
pub enum DefinitionResourceTypes {}
pub enum RelatedClaimRelationship {}
pub enum PaymentAdjustmentReason {}
pub enum QuestionMaxOccurs {}
pub enum ModifiedFoodtype {}
pub enum Icd10Procedures {}
pub enum AdverseEventSeriousness {
    SeriousResultsInDeath(Option<Element>),
    SeriousIsLifeThreatening(Option<Element>),
    SeriousResultsInHospitalization(Option<Element>),
    SeriousResultsInDisability(Option<Element>),
    SeriousIsBirthDefect(Option<Element>),
    SeriousRequiresPreventImpairment(Option<Element>),
}
pub enum ReasonMedicationGivenCodes {}
pub enum DetectedissueSeverity {}
pub enum MissingToothReason {}
pub enum VisionProduct {}
pub enum ReactionEventSeverity {}
pub enum ChargeitemStatus {}
pub enum MessageReasonEncounter {}
pub enum ContractPartyRole {}
pub enum EndpointStatus {}
pub enum V3RoleClassManufacturedProduct {
    _RoleClassAssociative(Option<Element>),
    _RoleClassRelationshipFormal(Option<Element>),
    ASSIGNED(Option<Element>),
    CON(Option<Element>),
    GUARD(Option<Element>),
    CLAIM(Option<Element>),
    NAMED(Option<Element>),
    PROG(Option<Element>),
    MIL(Option<Element>),
    CASEBJ(Option<Element>),
    RESBJ(Option<Element>),
    NOT(Option<Element>),
    PROV(Option<Element>),
    CAREGIVER(Option<Element>),
    PRS(Option<Element>),
    SELF(Option<Element>),
    ACCESS(Option<Element>),
    ADJY(Option<Element>),
    BOND(Option<Element>),
    CONY(Option<Element>),
    ADMM(Option<Element>),
    BIRTHPL(Option<Element>),
    DEATHPLC(Option<Element>),
    DST(Option<Element>),
    EXLOC(Option<Element>),
    DSDLOC(Option<Element>),
    ISDLOC(Option<Element>),
    EXPR(Option<Element>),
    HLD(Option<Element>),
    HLTHCHRT(Option<Element>),
    IDENT(Option<Element>),
    MANU(Option<Element>),
    MNT(Option<Element>),
    OWN(Option<Element>),
    RGPR(Option<Element>),
    TERR(Option<Element>),
    USED(Option<Element>),
    WRTE(Option<Element>),
    _RoleClassOntological(Option<Element>),
    SAME(Option<Element>),
    SUBY(Option<Element>),
    GRIC(Option<Element>),
    _RoleClassPartitive(Option<Element>),
    EXPVECTOR(Option<Element>),
    FOMITE(Option<Element>),
    ACTI(Option<Element>),
    ADJV(Option<Element>),
    ADTV(Option<Element>),
    BASE(Option<Element>),
    CNTM(Option<Element>),
    IACT(Option<Element>),
    MECH(Option<Element>),
    STOR(Option<Element>),
    ACTM(Option<Element>),
    ALQT(Option<Element>),
    ISLT(Option<Element>),
}
pub enum ActivityDefinitionCategory {}
pub enum EncounterSpecialCourtesy {}
pub enum EncounterDiet {}
pub enum RequestResourceTypes {}
pub enum ContractPublicationstatus {}
pub enum SupplydeliveryStatus {}
pub enum V3ProbabilityDistributionType {}
pub enum ActionSelectionBehavior {}
pub enum CompositeMeasureScoring {}
pub enum ListItemFlag {}
pub enum V3TableRules {}
pub enum SearchComparator {}
pub enum MedAdminPerformFunction {}
pub enum V3TableCellHorizontalAlign {}
pub enum AuditEntityType {}
pub enum V3DocumentCompletion {}
pub enum EvidenceQuality {}
pub enum MedicationrequestStatus {}
pub enum OralProsthodonticMaterial {}
pub enum MedicationknowledgeCharacteristic {}
pub enum EncounterAdmitSource {}
pub enum AdministrativeGender {}
pub enum ServiceReferralMethod {}
pub enum ContactentityType {}
pub enum V3TableFrame {}
pub enum V3QueryParameterValue {
    ALLDISP(Option<Element>),
    LASTDISP(Option<Element>),
    NODISP(Option<Element>),
    AO(Option<Element>),
    ONR(Option<Element>),
    OWR(Option<Element>),
    C(Option<Element>),
    N(Option<Element>),
    R(Option<Element>),
    ISSFA(Option<Element>),
    ISSFI(Option<Element>),
    ISSFU(Option<Element>),
}
pub enum RiskProbability {}
pub enum V3LocalRemoteControlState {}
pub enum MetricOperationalStatus {}
pub enum HandlingCondition {}
pub enum ContractScope {}
pub enum ImmunizationEvaluationTargetDisease {}
pub enum ContractStatus {}
pub enum VersioningPolicy {}
pub enum ServiceUscls {}
pub enum ConsentAction {}
pub enum ResearchStudyObjectiveType {}
pub enum V3ContextControl {
    AN(Option<Element>),
    AP(Option<Element>),
    ON(Option<Element>),
    OP(Option<Element>),
}
pub enum V3EducationLevel {}
pub enum AuditEventOutcome {}
pub enum ImmunizationRecommendationStatus {}
pub enum OperationKind {}
pub enum V3RoleClassAgent {
    _RoleClassAssociative(Option<Element>),
    _RoleClassRelationshipFormal(Option<Element>),
    ASSIGNED(Option<Element>),
    CON(Option<Element>),
    GUARD(Option<Element>),
    CLAIM(Option<Element>),
    NAMED(Option<Element>),
    PROG(Option<Element>),
    MIL(Option<Element>),
    CASEBJ(Option<Element>),
    RESBJ(Option<Element>),
    NOT(Option<Element>),
    PROV(Option<Element>),
    CAREGIVER(Option<Element>),
    PRS(Option<Element>),
    SELF(Option<Element>),
    ACCESS(Option<Element>),
    ADJY(Option<Element>),
    BOND(Option<Element>),
    CONY(Option<Element>),
    ADMM(Option<Element>),
    BIRTHPL(Option<Element>),
    DEATHPLC(Option<Element>),
    DST(Option<Element>),
    EXLOC(Option<Element>),
    DSDLOC(Option<Element>),
    ISDLOC(Option<Element>),
    EXPR(Option<Element>),
    HLD(Option<Element>),
    HLTHCHRT(Option<Element>),
    IDENT(Option<Element>),
    MANU(Option<Element>),
    MNT(Option<Element>),
    OWN(Option<Element>),
    RGPR(Option<Element>),
    TERR(Option<Element>),
    USED(Option<Element>),
    WRTE(Option<Element>),
    _RoleClassOntological(Option<Element>),
    SAME(Option<Element>),
    SUBY(Option<Element>),
    GRIC(Option<Element>),
    _RoleClassPartitive(Option<Element>),
    EXPVECTOR(Option<Element>),
    FOMITE(Option<Element>),
    ACTI(Option<Element>),
    ADJV(Option<Element>),
    ADTV(Option<Element>),
    BASE(Option<Element>),
    CNTM(Option<Element>),
    IACT(Option<Element>),
    MECH(Option<Element>),
    STOR(Option<Element>),
    ACTM(Option<Element>),
    ALQT(Option<Element>),
    ISLT(Option<Element>),
}
pub enum V3EntityNameUseR2 {
    A(Option<Element>),
    ANON(Option<Element>),
    I(Option<Element>),
    P(Option<Element>),
    R(Option<Element>),
    ABC(Option<Element>),
    IDE(Option<Element>),
    SYL(Option<Element>),
    DN(Option<Element>),
}
pub enum ImmunizationEvaluationDoseStatus {}
pub enum V3ActRelationshipSubset {
    FUTURE(Option<Element>),
    PAST(Option<Element>),
    SUM(Option<Element>),
}
pub enum V3QueryResponse {}
pub enum ProductCategory {}
pub enum ObservationVitalsignresult {}
pub enum UdiEntryType {}
pub enum LocationPhysicalType {}
pub enum ServicePlace {}
pub enum VerificationresultCanPushUpdates {}
pub enum VerificationresultValidationType {}
pub enum V3ActRelationshipPertains {
    _ActClassTemporallyPertains(Option<Element>),
    _ActRelationshipAccounting(Option<Element>),
    CHRG(Option<Element>),
    COST(Option<Element>),
    CREDIT(Option<Element>),
    DEBIT(Option<Element>),
    _ActRelationshipConditional(Option<Element>),
    BLOCK(Option<Element>),
    DIAG(Option<Element>),
    IMM(Option<Element>),
    MITGT(Option<Element>),
    PRYLX(Option<Element>),
    TREAT(Option<Element>),
    _ActRelationshipTemporallyPertains(Option<Element>),
    ENE(Option<Element>),
    CONCURRENT(Option<Element>),
    SBSECWE(Option<Element>),
    ENS(Option<Element>),
    SNE(Option<Element>),
    SNS(Option<Element>),
    SCWSEBE(Option<Element>),
    SCWSEAE(Option<Element>),
    EAE(Option<Element>),
    SBEEAE(Option<Element>),
    SAS(Option<Element>),
    EAEORECW(Option<Element>),
    OVERLAP(Option<Element>),
    SBSEASEBE(Option<Element>),
    SBE(Option<Element>),
    SBSEBE(Option<Element>),
    EBS(Option<Element>),
    SBS(Option<Element>),
    AUTH(Option<Element>),
    CAUS(Option<Element>),
    COMP(Option<Element>),
    STEP(Option<Element>),
    COVBY(Option<Element>),
    DRIV(Option<Element>),
    ELNK(Option<Element>),
    EVID(Option<Element>),
    EXACBY(Option<Element>),
    EXPL(Option<Element>),
    INTF(Option<Element>),
    ITEMSLOC(Option<Element>),
    LIMIT(Option<Element>),
    META(Option<Element>),
    MFST(Option<Element>),
    NAME(Option<Element>),
    OUTC(Option<Element>),
    OBJC(Option<Element>),
    OBJF(Option<Element>),
    PERT(Option<Element>),
    PREV(Option<Element>),
    REFR(Option<Element>),
    REFV(Option<Element>),
    RELVBY(Option<Element>),
    SEQL(Option<Element>),
    OCCR(Option<Element>),
    OREF(Option<Element>),
    SCH(Option<Element>),
    VRXCRPT(Option<Element>),
    SPRT(Option<Element>),
    SUBJ(Option<Element>),
    SUMM(Option<Element>),
    VALUE(Option<Element>),
}
pub enum ConstraintSeverity {}
pub enum FmStatus {}
pub enum GoalRelationshipType {}
pub enum MessageEvents {}
pub enum V3ParticipationVerifier {
    _ParticipationAncillary(Option<Element>),
    _ParticipationInformationGenerator(Option<Element>),
    ENT(Option<Element>),
    CST(Option<Element>),
    DIR(Option<Element>),
    TPA(Option<Element>),
    NRD(Option<Element>),
    RDV(Option<Element>),
    EXPTRGT(Option<Element>),
    EXSRC(Option<Element>),
    SPC(Option<Element>),
    IND(Option<Element>),
    IRCP(Option<Element>),
    LOC(Option<Element>),
    PRF(Option<Element>),
    RESP(Option<Element>),
    VRF(Option<Element>),
}
pub enum ClaimUse {}
pub enum AddressType {}
pub enum ResponseCode {}
pub enum ImmunizationOrigin {}
pub enum SearchModifierCode {}
pub enum ConsentDataMeaning {}
pub enum ConsentProvisionType {}
pub enum UcumVitalsCommon {}
pub enum ClaimSubtype {}
pub enum ProductStatus {}
pub enum Participantrequired {}
pub enum TextureCode {}
pub enum MedicationStatus {}
pub enum ListMode {}
pub enum AbstractTypes {}
pub enum GroupMeasure {}
pub enum PublicationStatus {}
pub enum V3TableCellVerticalAlign {}
pub enum V3Calendar {}
pub enum PropertyRepresentation {}
pub enum AdverseEventCausalityMethod {}
pub enum AllergyintoleranceClinical {
    Resolved(Option<Element>),
}
pub enum V3RoleClassServiceDeliveryLocation {
    _RoleClassAssociative(Option<Element>),
    _RoleClassRelationshipFormal(Option<Element>),
    ASSIGNED(Option<Element>),
    CON(Option<Element>),
    GUARD(Option<Element>),
    CLAIM(Option<Element>),
    NAMED(Option<Element>),
    PROG(Option<Element>),
    MIL(Option<Element>),
    CASEBJ(Option<Element>),
    RESBJ(Option<Element>),
    NOT(Option<Element>),
    PROV(Option<Element>),
    CAREGIVER(Option<Element>),
    PRS(Option<Element>),
    SELF(Option<Element>),
    ACCESS(Option<Element>),
    ADJY(Option<Element>),
    BOND(Option<Element>),
    CONY(Option<Element>),
    ADMM(Option<Element>),
    BIRTHPL(Option<Element>),
    DEATHPLC(Option<Element>),
    DST(Option<Element>),
    EXLOC(Option<Element>),
    DSDLOC(Option<Element>),
    ISDLOC(Option<Element>),
    EXPR(Option<Element>),
    HLD(Option<Element>),
    HLTHCHRT(Option<Element>),
    IDENT(Option<Element>),
    MANU(Option<Element>),
    MNT(Option<Element>),
    OWN(Option<Element>),
    RGPR(Option<Element>),
    TERR(Option<Element>),
    USED(Option<Element>),
    WRTE(Option<Element>),
    _RoleClassOntological(Option<Element>),
    SAME(Option<Element>),
    SUBY(Option<Element>),
    GRIC(Option<Element>),
    _RoleClassPartitive(Option<Element>),
    EXPVECTOR(Option<Element>),
    FOMITE(Option<Element>),
    ACTI(Option<Element>),
    ADJV(Option<Element>),
    ADTV(Option<Element>),
    BASE(Option<Element>),
    CNTM(Option<Element>),
    IACT(Option<Element>),
    MECH(Option<Element>),
    STOR(Option<Element>),
    ACTM(Option<Element>),
    ALQT(Option<Element>),
    ISLT(Option<Element>),
}
pub enum AdverseEventActuality {}
pub enum BasicResourceType {}
pub enum V3ParticipationMode {
    DICTATE(Option<Element>),
    FACE(Option<Element>),
    PHONE(Option<Element>),
    VIDEOCONF(Option<Element>),
    FAXWRIT(Option<Element>),
    HANDWRIT(Option<Element>),
    MAILWRIT(Option<Element>),
    ONLINEWRIT(Option<Element>),
    TYPEWRIT(Option<Element>),
}
pub enum SupplyrequestStatus {}
pub enum V3RoleLinkType {
    BACKUP(Option<Element>),
    CONT(Option<Element>),
    DIRAUTH(Option<Element>),
    IDENT(Option<Element>),
    INDAUTH(Option<Element>),
    PART(Option<Element>),
    REPL(Option<Element>),
}
pub enum ConditionState {}
pub enum ImmunizationSite {}
pub enum DocumentMode {}
pub enum AccountStatus {}
pub enum ContractActorrole {}
pub enum ClaimCareteamrole {}
pub enum CarePlanActivityKind {}
pub enum ConsistencyType {}
pub enum ServiceProvisionConditions {}
pub enum AllergyIntoleranceCategory {}
pub enum AnimalSpecies {}
pub enum V3RoleClassAssociative {
    _RoleClassAssociative(Option<Element>),
    _RoleClassRelationshipFormal(Option<Element>),
    ASSIGNED(Option<Element>),
    CON(Option<Element>),
    GUARD(Option<Element>),
    CLAIM(Option<Element>),
    NAMED(Option<Element>),
    PROG(Option<Element>),
    MIL(Option<Element>),
    CASEBJ(Option<Element>),
    RESBJ(Option<Element>),
    NOT(Option<Element>),
    PROV(Option<Element>),
    CAREGIVER(Option<Element>),
    PRS(Option<Element>),
    SELF(Option<Element>),
    ACCESS(Option<Element>),
    ADJY(Option<Element>),
    BOND(Option<Element>),
    CONY(Option<Element>),
    ADMM(Option<Element>),
    BIRTHPL(Option<Element>),
    DEATHPLC(Option<Element>),
    DST(Option<Element>),
    EXLOC(Option<Element>),
    DSDLOC(Option<Element>),
    ISDLOC(Option<Element>),
    EXPR(Option<Element>),
    HLD(Option<Element>),
    HLTHCHRT(Option<Element>),
    IDENT(Option<Element>),
    MANU(Option<Element>),
    MNT(Option<Element>),
    OWN(Option<Element>),
    RGPR(Option<Element>),
    TERR(Option<Element>),
    USED(Option<Element>),
    WRTE(Option<Element>),
    _RoleClassOntological(Option<Element>),
    SAME(Option<Element>),
    SUBY(Option<Element>),
    GRIC(Option<Element>),
    _RoleClassPartitive(Option<Element>),
    EXPVECTOR(Option<Element>),
    FOMITE(Option<Element>),
    ACTI(Option<Element>),
    ADJV(Option<Element>),
    ADTV(Option<Element>),
    BASE(Option<Element>),
    CNTM(Option<Element>),
    IACT(Option<Element>),
    MECH(Option<Element>),
    STOR(Option<Element>),
    ACTM(Option<Element>),
    ALQT(Option<Element>),
    ISLT(Option<Element>),
}
pub enum TaskCode {}
pub enum ExPaymenttype {}
pub enum ConformanceExpectation {}
pub enum ResearchStudyStatus {}
pub enum V3PersonDisabilityType {
    CB(Option<Element>),
    CR(Option<Element>),
    G(Option<Element>),
    WC(Option<Element>),
    WK(Option<Element>),
}
pub enum ImmunizationRoute {}
pub enum EncounterStatus {}
pub enum V3SetOperator {
    E(Option<Element>),
    I(Option<Element>),
}
pub enum V3RoleClassRelationshipFormal {
    _RoleClassAssociative(Option<Element>),
    _RoleClassRelationshipFormal(Option<Element>),
    ASSIGNED(Option<Element>),
    CON(Option<Element>),
    GUARD(Option<Element>),
    CLAIM(Option<Element>),
    NAMED(Option<Element>),
    PROG(Option<Element>),
    MIL(Option<Element>),
    CASEBJ(Option<Element>),
    RESBJ(Option<Element>),
    NOT(Option<Element>),
    PROV(Option<Element>),
    CAREGIVER(Option<Element>),
    PRS(Option<Element>),
    SELF(Option<Element>),
    ACCESS(Option<Element>),
    ADJY(Option<Element>),
    BOND(Option<Element>),
    CONY(Option<Element>),
    ADMM(Option<Element>),
    BIRTHPL(Option<Element>),
    DEATHPLC(Option<Element>),
    DST(Option<Element>),
    EXLOC(Option<Element>),
    DSDLOC(Option<Element>),
    ISDLOC(Option<Element>),
    EXPR(Option<Element>),
    HLD(Option<Element>),
    HLTHCHRT(Option<Element>),
    IDENT(Option<Element>),
    MANU(Option<Element>),
    MNT(Option<Element>),
    OWN(Option<Element>),
    RGPR(Option<Element>),
    TERR(Option<Element>),
    USED(Option<Element>),
    WRTE(Option<Element>),
    _RoleClassOntological(Option<Element>),
    SAME(Option<Element>),
    SUBY(Option<Element>),
    GRIC(Option<Element>),
    _RoleClassPartitive(Option<Element>),
    EXPVECTOR(Option<Element>),
    FOMITE(Option<Element>),
    ACTI(Option<Element>),
    ADJV(Option<Element>),
    ADTV(Option<Element>),
    BASE(Option<Element>),
    CNTM(Option<Element>),
    IACT(Option<Element>),
    MECH(Option<Element>),
    STOR(Option<Element>),
    ACTM(Option<Element>),
    ALQT(Option<Element>),
    ISLT(Option<Element>),
}
pub enum V3DeviceAlertLevel {}
pub enum ContractSecurityClassification {}
pub enum V3RoleClassSpecimen {
    _RoleClassAssociative(Option<Element>),
    _RoleClassRelationshipFormal(Option<Element>),
    ASSIGNED(Option<Element>),
    CON(Option<Element>),
    GUARD(Option<Element>),
    CLAIM(Option<Element>),
    NAMED(Option<Element>),
    PROG(Option<Element>),
    MIL(Option<Element>),
    CASEBJ(Option<Element>),
    RESBJ(Option<Element>),
    NOT(Option<Element>),
    PROV(Option<Element>),
    CAREGIVER(Option<Element>),
    PRS(Option<Element>),
    SELF(Option<Element>),
    ACCESS(Option<Element>),
    ADJY(Option<Element>),
    BOND(Option<Element>),
    CONY(Option<Element>),
    ADMM(Option<Element>),
    BIRTHPL(Option<Element>),
    DEATHPLC(Option<Element>),
    DST(Option<Element>),
    EXLOC(Option<Element>),
    DSDLOC(Option<Element>),
    ISDLOC(Option<Element>),
    EXPR(Option<Element>),
    HLD(Option<Element>),
    HLTHCHRT(Option<Element>),
    IDENT(Option<Element>),
    MANU(Option<Element>),
    MNT(Option<Element>),
    OWN(Option<Element>),
    RGPR(Option<Element>),
    TERR(Option<Element>),
    USED(Option<Element>),
    WRTE(Option<Element>),
    _RoleClassOntological(Option<Element>),
    SAME(Option<Element>),
    SUBY(Option<Element>),
    GRIC(Option<Element>),
    _RoleClassPartitive(Option<Element>),
    EXPVECTOR(Option<Element>),
    FOMITE(Option<Element>),
    ACTI(Option<Element>),
    ADJV(Option<Element>),
    ADTV(Option<Element>),
    BASE(Option<Element>),
    CNTM(Option<Element>),
    IACT(Option<Element>),
    MECH(Option<Element>),
    STOR(Option<Element>),
    ACTM(Option<Element>),
    ALQT(Option<Element>),
    ISLT(Option<Element>),
}
pub enum ImmunizationSubpotentReason {}
pub enum RemittanceOutcome {}
pub enum ResourceStatus {}
pub enum ActionCardinalityBehavior {}
pub enum GroupType {}
pub enum TypeDerivationRule {}
pub enum CarePlanActivityStatus {
    Stopped(Option<Element>),
}
pub enum QuestionnaireDisplayCategory {}
pub enum GraphCompartmentRule {}
pub enum Icd10 {}
pub enum UcumBodyweight {}
pub enum ExProcedureType {}
pub enum GuidanceResponseStatus {}
pub enum DocumentRelationshipType {}
pub enum CompartmentType {}
pub enum ContractExpirationType {}
pub enum CoverageCopayType {}
pub enum Adjudication {}
pub enum EffectEstimateType {}
pub enum InsuranceplanType {}
pub enum ContainerCap {}
pub enum FinancialTaskcode {}
pub enum MeasureImprovementNotation {}
pub enum MedicationdispensePerformerFunction {}
pub enum V3PaymentTerms {}
pub enum UnknownContentCode {}
pub enum V3Charset {}
pub enum FilterOperator {}
pub enum GuideParameterCode {}
pub enum MatchGrade {}
pub enum DataAbsentReason {
    AskedUnknown(Option<Element>),
    TempUnknown(Option<Element>),
    NotAsked(Option<Element>),
    AskedDeclined(Option<Element>),
    NotANumber(Option<Element>),
    NegativeInfinity(Option<Element>),
    PositiveInfinity(Option<Element>),
}
pub enum V3ParticipationTargetDirect {
    _ParticipationAncillary(Option<Element>),
    _ParticipationInformationGenerator(Option<Element>),
    ENT(Option<Element>),
    CST(Option<Element>),
    DIR(Option<Element>),
    TPA(Option<Element>),
    NRD(Option<Element>),
    RDV(Option<Element>),
    EXPTRGT(Option<Element>),
    EXSRC(Option<Element>),
    SPC(Option<Element>),
    IND(Option<Element>),
    IRCP(Option<Element>),
    LOC(Option<Element>),
    PRF(Option<Element>),
    RESP(Option<Element>),
    VRF(Option<Element>),
}
pub enum V3VerificationMethod {
    ALGM(Option<Element>),
    GINT(Option<Element>),
    PCR(Option<Element>),
    AVERAGE(Option<Element>),
    COUNT(Option<Element>),
    MAX(Option<Element>),
    MEDIAN(Option<Element>),
    MIN(Option<Element>),
    MODE(Option<Element>),
    STDEVP(Option<Element>),
    STDEVS(Option<Element>),
    SUM(Option<Element>),
    VARIANCEP(Option<Element>),
    VARIANCES(Option<Element>),
    VDOC(Option<Element>),
    VREG(Option<Element>),
    VTOKEN(Option<Element>),
    VVOICE(Option<Element>),
    V0240(Option<Element>),
    V0241(Option<Element>),
    V0242(Option<Element>),
    V0272(Option<Element>),
    V0273(Option<Element>),
    V0274(Option<Element>),
    V0275(Option<Element>),
    V0275a(Option<Element>),
    V0276(Option<Element>),
    V0277(Option<Element>),
    V0278(Option<Element>),
    V0279(Option<Element>),
}
pub enum NoteType {}
pub enum SecondaryFinding {}
pub enum ConditionCategory {}
pub enum NetworkType {}
pub enum EncounterDischargeDisposition {}
pub enum V3ParticipationTargetSubject {
    _ParticipationAncillary(Option<Element>),
    _ParticipationInformationGenerator(Option<Element>),
    ENT(Option<Element>),
    CST(Option<Element>),
    DIR(Option<Element>),
    TPA(Option<Element>),
    NRD(Option<Element>),
    RDV(Option<Element>),
    EXPTRGT(Option<Element>),
    EXSRC(Option<Element>),
    SPC(Option<Element>),
    IND(Option<Element>),
    IRCP(Option<Element>),
    LOC(Option<Element>),
    PRF(Option<Element>),
    RESP(Option<Element>),
    VRF(Option<Element>),
}
pub enum V3EntityClass {
    HCE(Option<Element>),
    LIV(Option<Element>),
    ANM(Option<Element>),
    MIC(Option<Element>),
    PLNT(Option<Element>),
    MAT(Option<Element>),
    CONT(Option<Element>),
    DEV(Option<Element>),
    ORG(Option<Element>),
    NAT(Option<Element>),
    PLC(Option<Element>),
    RGRP(Option<Element>),
}
pub enum V3EntityStatus {
    Active(Option<Element>),
    Inactive(Option<Element>),
    Terminated(Option<Element>),
}
pub enum ExDiagnosisOnAdmission {}
pub enum ImmunizationEvaluationDoseStatusReason {}
pub enum SubjectType {}
pub enum V3ActRelationshipJoin {}
pub enum SortDirection {}
pub enum ConditionalDeleteStatus {}
pub enum V3ActClassDocument {
    _ActClassRecordOrganizer(Option<Element>),
    DOC(Option<Element>),
    CDALVLONE(Option<Element>),
    CATEGORY(Option<Element>),
    DOCBODY(Option<Element>),
    DOCSECT(Option<Element>),
    TOPIC(Option<Element>),
    EHR(Option<Element>),
    CLUSTER(Option<Element>),
    ACCM(Option<Element>),
    ACCT(Option<Element>),
    ACSN(Option<Element>),
    ADJUD(Option<Element>),
    CACT(Option<Element>),
    CNTRCT(Option<Element>),
    COV(Option<Element>),
    CONC(Option<Element>),
    CONS(Option<Element>),
    CONTREG(Option<Element>),
    CTTEVENT(Option<Element>),
    DISPACT(Option<Element>),
    EXPOS(Option<Element>),
    INC(Option<Element>),
    INFRM(Option<Element>),
    INVE(Option<Element>),
    LIST(Option<Element>),
    MPROT(Option<Element>),
    OBS(Option<Element>),
    ROIBND(Option<Element>),
    ROIOVL(Option<Element>),
    _SubjectBodyPosition(Option<Element>),
    RTRD(Option<Element>),
    TRD(Option<Element>),
    CASE(Option<Element>),
    DETPOL(Option<Element>),
    EXP(Option<Element>),
    LOC(Option<Element>),
    PHN(Option<Element>),
    POL(Option<Element>),
    SEQ(Option<Element>),
    SEQVAR(Option<Element>),
    OBSCOR(Option<Element>),
    POSACC(Option<Element>),
    POSCOORD(Option<Element>),
    PCPR(Option<Element>),
    POLICY(Option<Element>),
    PROC(Option<Element>),
    SPECCOLLECT(Option<Element>),
    REG(Option<Element>),
    REV(Option<Element>),
    SPCTRT(Option<Element>),
    SPLY(Option<Element>),
    STORE(Option<Element>),
    SUBST(Option<Element>),
    TRFR(Option<Element>),
    TRNS(Option<Element>),
    XACT(Option<Element>),
    _ActClassContainer(Option<Element>),
}
pub enum SignatureType {}
pub enum V3TelecommunicationCapabilities {}
pub enum QuestionnaireEnableOperator {}
pub enum V3MapRelationship {}
pub enum MapTargetListMode {}
pub enum ReportStatusCodes {}
pub enum ContractSecurityCategory {}
pub enum V3Hl7CMETAttribution {}
pub enum V3RoleClass {
    _RoleClassAssociative(Option<Element>),
    _RoleClassRelationshipFormal(Option<Element>),
    ASSIGNED(Option<Element>),
    CON(Option<Element>),
    GUARD(Option<Element>),
    CLAIM(Option<Element>),
    NAMED(Option<Element>),
    PROG(Option<Element>),
    MIL(Option<Element>),
    CASEBJ(Option<Element>),
    RESBJ(Option<Element>),
    NOT(Option<Element>),
    PROV(Option<Element>),
    CAREGIVER(Option<Element>),
    PRS(Option<Element>),
    SELF(Option<Element>),
    ACCESS(Option<Element>),
    ADJY(Option<Element>),
    BOND(Option<Element>),
    CONY(Option<Element>),
    ADMM(Option<Element>),
    BIRTHPL(Option<Element>),
    DEATHPLC(Option<Element>),
    DST(Option<Element>),
    EXLOC(Option<Element>),
    DSDLOC(Option<Element>),
    ISDLOC(Option<Element>),
    EXPR(Option<Element>),
    HLD(Option<Element>),
    HLTHCHRT(Option<Element>),
    IDENT(Option<Element>),
    MANU(Option<Element>),
    MNT(Option<Element>),
    OWN(Option<Element>),
    RGPR(Option<Element>),
    TERR(Option<Element>),
    USED(Option<Element>),
    WRTE(Option<Element>),
    _RoleClassOntological(Option<Element>),
    SAME(Option<Element>),
    SUBY(Option<Element>),
    GRIC(Option<Element>),
    _RoleClassPartitive(Option<Element>),
    EXPVECTOR(Option<Element>),
    FOMITE(Option<Element>),
    ACTI(Option<Element>),
    ADJV(Option<Element>),
    ADTV(Option<Element>),
    BASE(Option<Element>),
    CNTM(Option<Element>),
    IACT(Option<Element>),
    MECH(Option<Element>),
    STOR(Option<Element>),
    ACTM(Option<Element>),
    ALQT(Option<Element>),
    ISLT(Option<Element>),
}
pub enum Relationship {}
pub enum AdjudicationReason {}
pub enum V3ReligiousAffiliation {}
pub enum EvidenceVariantState {}
pub enum VariableType {}
pub enum SubscriberRelationship {}
pub enum ResourceValidationMode {}
pub enum UcumCommon {}
pub enum MedicationknowledgeStatus {}
pub enum MeasureType {}
pub enum SynthesisType {}
pub enum ContributorType {}
pub enum ProcedureCategory {}
pub enum ConceptmapUnmappedMode {}
pub enum ContractSignerType {}
pub enum Dicm405Mediatype {}
pub enum OrientationType {}
pub enum VerificationresultStatus {}
pub enum AllergyIntoleranceType {}
pub enum FHIRVersion {}
pub enum MeasureDataUsage {}
pub enum ObservationRangeCategory {}
pub enum ImmunizationProgramEligibility {}
pub enum ProcedureProgressStatusCodes {}
pub enum LocationStatus {}
pub enum AuditEventAction {}
pub enum QuantityComparator {}
pub enum V3OrderableDrugForm {
    APPFUL(Option<Element>),
    DROP(Option<Element>),
    PUFF(Option<Element>),
    SCOOP(Option<Element>),
    SPRY(Option<Element>),
    _GasDrugForm(Option<Element>),
    _GasLiquidMixture(Option<Element>),
    BAINHL(Option<Element>),
    INHLSOL(Option<Element>),
    MDINHL(Option<Element>),
    NASSPRY(Option<Element>),
    FOAMAPL(Option<Element>),
    RECFORM(Option<Element>),
    VAGFOAM(Option<Element>),
    _GasSolidSpray(Option<Element>),
    BAINHLPWD(Option<Element>),
    INHLPWD(Option<Element>),
    MDINHLPWD(Option<Element>),
    NASINHL(Option<Element>),
    ORINHL(Option<Element>),
    _Liquid(Option<Element>),
    LIQSOAP(Option<Element>),
    SHMP(Option<Element>),
    TOPOIL(Option<Element>),
    IPSOL(Option<Element>),
    IRSOL(Option<Element>),
    IVSOL(Option<Element>),
    ORALSOL(Option<Element>),
    RECSOL(Option<Element>),
    TOPSOL(Option<Element>),
    _LiquidLiquidEmulsion(Option<Element>),
    NASCRM(Option<Element>),
    OPCRM(Option<Element>),
    ORCRM(Option<Element>),
    OTCRM(Option<Element>),
    RECCRM(Option<Element>),
    TOPCRM(Option<Element>),
    VAGCRM(Option<Element>),
    TOPLTN(Option<Element>),
    NASOINT(Option<Element>),
    OINTAPL(Option<Element>),
    OPOINT(Option<Element>),
    OTOINT(Option<Element>),
    RECOINT(Option<Element>),
    TOPOINT(Option<Element>),
    VAGOINT(Option<Element>),
    _LiquidSolidSuspension(Option<Element>),
    GELAPL(Option<Element>),
    NASGEL(Option<Element>),
    OPGEL(Option<Element>),
    OTGEL(Option<Element>),
    TOPGEL(Option<Element>),
    URETHGEL(Option<Element>),
    VAGGEL(Option<Element>),
    PUD(Option<Element>),
    TPASTE(Option<Element>),
    ITSUSP(Option<Element>),
    OPSUSP(Option<Element>),
    ORSUSP(Option<Element>),
    ERSUSP12(Option<Element>),
    ERSUSP24(Option<Element>),
    OTSUSP(Option<Element>),
    RECSUSP(Option<Element>),
    _SolidDrugForm(Option<Element>),
    BARSOAP(Option<Element>),
    CHEWBAR(Option<Element>),
    MEDPAD(Option<Element>),
    TPATCH(Option<Element>),
    CAP(Option<Element>),
    ENTCAP(Option<Element>),
    ERCAP(Option<Element>),
    TAB(Option<Element>),
    BUCTAB(Option<Element>),
    CAPLET(Option<Element>),
    CHEWTAB(Option<Element>),
    CPTAB(Option<Element>),
    DISINTAB(Option<Element>),
    DRTAB(Option<Element>),
    ECTAB(Option<Element>),
    ERTAB(Option<Element>),
    ORTROCHE(Option<Element>),
    SLTAB(Option<Element>),
    TOPPWD(Option<Element>),
    RECSUPP(Option<Element>),
    URETHSUPP(Option<Element>),
    VAGSUPP(Option<Element>),
    MEDSWAB(Option<Element>),
}
pub enum V3AddressPartType {
    DAL(Option<Element>),
    SAL(Option<Element>),
}
pub enum FlagStatus {}
pub enum MeasureReportStatus {}
pub enum MetricColor {}
pub enum ActionRelationshipType {}
pub enum DurationUnits {}
pub enum DeviceComponentProperty {}
pub enum ClaimInformationcategory {}
pub enum BenefitNetwork {}
pub enum ObservationStatus {
    Corrected(Option<Element>),
}
pub enum CompositionAltcodeKind {}
pub enum SearchParamType {}
pub enum V3QueryRequestLimit {
    RD(Option<Element>),
}
pub enum Hl7WorkGroup {}
pub enum MedicationdispenseCategory {}
pub enum CoverageSelfpay {}
pub enum FocalSubject {}
pub enum MedicationrequestCourseOfTherapy {}
pub enum InvestigationSets {}
pub enum ConsentPolicy {}
pub enum GuidePageGeneration {}
pub enum ObservationCategory {}
pub enum V3Hl7ITSVersionCode {}
pub enum V3ResponseLevel {}
pub enum V3TableCellScope {}
pub enum V3ContainerSeparator {}
pub enum ConsentScope {}
pub enum ProviderQualification {}
pub enum ConsentPerformer {}
pub enum MapTransform {}
pub enum V3Sequencing {}
pub enum OperationOutcome {}
pub enum VariantState {}
pub enum LinkType {}
pub enum HttpVerb {}
pub enum MessageTransport {}
pub enum EncounterType {}
pub enum ConceptSubsumptionOutcome {}
pub enum ClaimModifiers {}
pub enum AdverseEventSeverity {}
pub enum MetricCategory {}
pub enum PaymentType {}
pub enum NamePartQualifier {}
pub enum InsuranceplanApplicability {}
pub enum CoverageClass {}
pub enum SmartCapabilities {}
pub enum ProvenanceHistoryRecordActivity {}
pub enum V3IdentifierReliability {}
pub enum EventResourceTypes {}
pub enum SubscriptionStatus {}
pub enum ProvenanceEntityRole {
    Revision(Option<Element>),
    Quotation(Option<Element>),
    Source(Option<Element>),
    Removal(Option<Element>),
}
pub enum AllergyintoleranceVerification {}
pub enum GoalAchievement {
    Improving(Option<Element>),
    Worsening(Option<Element>),
    NoChange(Option<Element>),
    Sustaining(Option<Element>),
    NoProgress(Option<Element>),
    NotAttainable(Option<Element>),
}
pub enum ProbabilityDistributionType {}
pub enum ReactionEventCertainty {}
pub enum CommunicationNotDoneReason {}
pub enum ObservationInterpretation {
    CAR(Option<Element>),
    Carrier(Option<Element>),
    B(Option<Element>),
    D(Option<Element>),
    U(Option<Element>),
    W(Option<Element>),
    Greater(Option<Element>),
    Less(Option<Element>),
    AC(Option<Element>),
    IE(Option<Element>),
    QCF(Option<Element>),
    TOX(Option<Element>),
    A(Option<Element>),
    HH(Option<Element>),
    LL(Option<Element>),
    HLess(Option<Element>),
    HU(Option<Element>),
    LGreater(Option<Element>),
    LU(Option<Element>),
    N(Option<Element>),
    I(Option<Element>),
    MS(Option<Element>),
    NCL(Option<Element>),
    NS(Option<Element>),
    R(Option<Element>),
    S(Option<Element>),
    VS(Option<Element>),
    HX(Option<Element>),
    LX(Option<Element>),
    IND(Option<Element>),
    NEG(Option<Element>),
    POS(Option<Element>),
    EXP(Option<Element>),
    UNE(Option<Element>),
    NR(Option<Element>),
    RR(Option<Element>),
}
pub enum ContractTermSubtype {}
pub enum ExpansionParameterSource {}
pub enum EligibilityrequestPurpose {}
pub enum Appointmentstatus {}
pub enum CertaintySubcomponentType {}
pub enum ContractSubtype {}
pub enum ConditionVerStatus {
    Provisional(Option<Element>),
    Differential(Option<Element>),
}
pub enum ParameterGroup {}
pub enum V3ActClassClinicalDocument {
    _ActClassRecordOrganizer(Option<Element>),
    DOC(Option<Element>),
    CDALVLONE(Option<Element>),
    CATEGORY(Option<Element>),
    DOCBODY(Option<Element>),
    DOCSECT(Option<Element>),
    TOPIC(Option<Element>),
    EHR(Option<Element>),
    CLUSTER(Option<Element>),
    ACCM(Option<Element>),
    ACCT(Option<Element>),
    ACSN(Option<Element>),
    ADJUD(Option<Element>),
    CACT(Option<Element>),
    CNTRCT(Option<Element>),
    COV(Option<Element>),
    CONC(Option<Element>),
    CONS(Option<Element>),
    CONTREG(Option<Element>),
    CTTEVENT(Option<Element>),
    DISPACT(Option<Element>),
    EXPOS(Option<Element>),
    INC(Option<Element>),
    INFRM(Option<Element>),
    INVE(Option<Element>),
    LIST(Option<Element>),
    MPROT(Option<Element>),
    OBS(Option<Element>),
    ROIBND(Option<Element>),
    ROIOVL(Option<Element>),
    _SubjectBodyPosition(Option<Element>),
    RTRD(Option<Element>),
    TRD(Option<Element>),
    CASE(Option<Element>),
    DETPOL(Option<Element>),
    EXP(Option<Element>),
    LOC(Option<Element>),
    PHN(Option<Element>),
    POL(Option<Element>),
    SEQ(Option<Element>),
    SEQVAR(Option<Element>),
    OBSCOR(Option<Element>),
    POSACC(Option<Element>),
    POSCOORD(Option<Element>),
    PCPR(Option<Element>),
    POLICY(Option<Element>),
    PROC(Option<Element>),
    SPECCOLLECT(Option<Element>),
    REG(Option<Element>),
    REV(Option<Element>),
    SPCTRT(Option<Element>),
    SPLY(Option<Element>),
    STORE(Option<Element>),
    SUBST(Option<Element>),
    TRFR(Option<Element>),
    TRNS(Option<Element>),
    XACT(Option<Element>),
    _ActClassContainer(Option<Element>),
}
pub enum V3SubstanceAdminSubstitution {
    E(Option<Element>),
    BC(Option<Element>),
    G(Option<Element>),
    TB(Option<Element>),
    TG(Option<Element>),
    F(Option<Element>),
    N(Option<Element>),
}
pub enum V3Hl7V3Conformance {}
pub enum V3ActClassProcedure {
    _ActClassRecordOrganizer(Option<Element>),
    DOC(Option<Element>),
    CDALVLONE(Option<Element>),
    CATEGORY(Option<Element>),
    DOCBODY(Option<Element>),
    DOCSECT(Option<Element>),
    TOPIC(Option<Element>),
    EHR(Option<Element>),
    CLUSTER(Option<Element>),
    ACCM(Option<Element>),
    ACCT(Option<Element>),
    ACSN(Option<Element>),
    ADJUD(Option<Element>),
    CACT(Option<Element>),
    CNTRCT(Option<Element>),
    COV(Option<Element>),
    CONC(Option<Element>),
    CONS(Option<Element>),
    CONTREG(Option<Element>),
    CTTEVENT(Option<Element>),
    DISPACT(Option<Element>),
    EXPOS(Option<Element>),
    INC(Option<Element>),
    INFRM(Option<Element>),
    INVE(Option<Element>),
    LIST(Option<Element>),
    MPROT(Option<Element>),
    OBS(Option<Element>),
    ROIBND(Option<Element>),
    ROIOVL(Option<Element>),
    _SubjectBodyPosition(Option<Element>),
    RTRD(Option<Element>),
    TRD(Option<Element>),
    CASE(Option<Element>),
    DETPOL(Option<Element>),
    EXP(Option<Element>),
    LOC(Option<Element>),
    PHN(Option<Element>),
    POL(Option<Element>),
    SEQ(Option<Element>),
    SEQVAR(Option<Element>),
    OBSCOR(Option<Element>),
    POSACC(Option<Element>),
    POSCOORD(Option<Element>),
    PCPR(Option<Element>),
    POLICY(Option<Element>),
    PROC(Option<Element>),
    SPECCOLLECT(Option<Element>),
    REG(Option<Element>),
    REV(Option<Element>),
    SPCTRT(Option<Element>),
    SPLY(Option<Element>),
    STORE(Option<Element>),
    SUBST(Option<Element>),
    TRFR(Option<Element>),
    TRNS(Option<Element>),
    XACT(Option<Element>),
    _ActClassContainer(Option<Element>),
}
pub enum ChargeitemBillingcodes {}
pub enum V3EntityDeterminer {
    GROUP(Option<Element>),
    GROUPKIND(Option<Element>),
    QUANTIFIED_KIND(Option<Element>),
}
pub enum ChromosomeHuman {}
pub enum ResearchElementType {}
pub enum ActionParticipantType {}
pub enum TaskIntent {}
pub enum ResourceAggregationMode {
    Bundled(Option<Element>),
}
pub enum RequestIntent {
    OriginalOrder(Option<Element>),
    ReflexOrder(Option<Element>),
    FillerOrder(Option<Element>),
}
pub enum ProductStorageScale {}
pub enum TriggerType {
    DataAdded(Option<Element>),
    DataModified(Option<Element>),
    DataRemoved(Option<Element>),
}
pub enum V3ObservationInterpretation {
    CAR(Option<Element>),
    Carrier(Option<Element>),
    B(Option<Element>),
    D(Option<Element>),
    U(Option<Element>),
    W(Option<Element>),
    Greater(Option<Element>),
    Less(Option<Element>),
    AC(Option<Element>),
    IE(Option<Element>),
    QCF(Option<Element>),
    TOX(Option<Element>),
    A(Option<Element>),
    HH(Option<Element>),
    LL(Option<Element>),
    HLess(Option<Element>),
    HU(Option<Element>),
    LGreater(Option<Element>),
    LU(Option<Element>),
    N(Option<Element>),
    I(Option<Element>),
    MS(Option<Element>),
    NCL(Option<Element>),
    NS(Option<Element>),
    R(Option<Element>),
    S(Option<Element>),
    VS(Option<Element>),
    HX(Option<Element>),
    LX(Option<Element>),
    IND(Option<Element>),
    NEG(Option<Element>),
    POS(Option<Element>),
    EXP(Option<Element>),
    UNE(Option<Element>),
    NR(Option<Element>),
    RR(Option<Element>),
}
pub enum V3CalendarCycle {
    CW(Option<Element>),
    CY(Option<Element>),
    D(Option<Element>),
    DW(Option<Element>),
    H(Option<Element>),
    M(Option<Element>),
    N(Option<Element>),
    S(Option<Element>),
    CD(Option<Element>),
    CH(Option<Element>),
    CM(Option<Element>),
    CN(Option<Element>),
    CS(Option<Element>),
    DY(Option<Element>),
    WY(Option<Element>),
}
pub enum ImmunizationRecommendationTargetDisease {}
pub enum GoalStartEvent {}
pub enum TemplateStatusCode {}
pub enum RequestPriority {}
pub enum SearchEntryMode {}
pub enum TypeRestfulInteraction {}
pub enum ExplanationofbenefitStatus {}
pub enum EndpointPayloadType {}
pub enum MedicationStatementStatus {}
pub enum MapGroupTypeMode {}
pub enum AgeUnits {}
pub enum V3ManagedParticipationStatus {
    Active(Option<Element>),
    Cancelled(Option<Element>),
    Completed(Option<Element>),
    Pending(Option<Element>),
}
pub enum LanguagePreferenceType {}
pub enum V3GeneralPurposeOfUse {}
pub enum EpisodeofcareType {}
pub enum ProvenanceAgentRole {
    Legal(Option<Element>),
}
pub enum ExPayeeResourceType {}
pub enum SubscriptionChannelType {}
pub enum V3RoleStatus {
    Active(Option<Element>),
    Cancelled(Option<Element>),
    Pending(Option<Element>),
    Suspended(Option<Element>),
    Terminated(Option<Element>),
}
pub enum TaskStatus {}
pub enum V3CodingRationale {}
pub enum V3ProvenanceEventCurrentState {}
pub enum V3AcknowledgementCondition {}
pub enum Teeth {}
pub enum DeviceStatementStatus {}
pub enum V3ContentProcessingMode {}
pub enum Intervention {}
pub enum V3EntityDeterminerDetermined {
    GROUP(Option<Element>),
    GROUPKIND(Option<Element>),
    QUANTIFIED_KIND(Option<Element>),
}
pub enum V3HtmlLinkType {}
pub enum V3LanguageAbilityProficiency {}
pub enum DataTypes {}
pub enum V3AcknowledgementDetailCode {
    NS200(Option<Element>),
    NS202(Option<Element>),
    NS203(Option<Element>),
    NS250(Option<Element>),
    NS260(Option<Element>),
    NS261(Option<Element>),
    SYN102(Option<Element>),
    SYN105(Option<Element>),
    SYN106(Option<Element>),
    SYN108(Option<Element>),
    SYN109(Option<Element>),
    SYN111(Option<Element>),
    SYN113(Option<Element>),
}
pub enum V3LivingArrangement {
    M(Option<Element>),
    T(Option<Element>),
    CS(Option<Element>),
    G(Option<Element>),
    N(Option<Element>),
    X(Option<Element>),
    H(Option<Element>),
    R(Option<Element>),
    SL(Option<Element>),
}
pub enum V3DataOperation {
    CREATE(Option<Element>),
    DELETE(Option<Element>),
    EXECUTE(Option<Element>),
    READ(Option<Element>),
    UPDATE(Option<Element>),
    ABORT(Option<Element>),
    ACTIVATE(Option<Element>),
    CANCEL(Option<Element>),
    COMPLETE(Option<Element>),
    HOLD(Option<Element>),
    JUMP(Option<Element>),
    NULLIFY(Option<Element>),
    OBSOLETE(Option<Element>),
    REACTIVATE(Option<Element>),
    RELEASE(Option<Element>),
    RESUME(Option<Element>),
    SUSPEND(Option<Element>),
}
pub enum V3ActMoodIntent {
    _ActMoodPotential(Option<Element>),
    EVN(Option<Element>),
    INT(Option<Element>),
    _ActMoodActRequest(Option<Element>),
    PRP(Option<Element>),
    APT(Option<Element>),
    CRT(Option<Element>),
    PRMSCRT(Option<Element>),
    RQOCRT(Option<Element>),
    EXPEC(Option<Element>),
    OPT(Option<Element>),
}
pub enum ContractAssettype {}
pub enum ExampleHierarchical {}
pub enum ImmunizationStatus {}
pub enum V3CompressionAlgorithm {}
pub enum DistanceUnits {}
pub enum V3ProcessingMode {}
pub enum DocSectionCodes {}
pub enum DeviceStatus {}
pub enum SequenceType {}
pub enum V3ResponseMode {}
pub enum V3ActClassSupply {
    _ActClassRecordOrganizer(Option<Element>),
    DOC(Option<Element>),
    CDALVLONE(Option<Element>),
    CATEGORY(Option<Element>),
    DOCBODY(Option<Element>),
    DOCSECT(Option<Element>),
    TOPIC(Option<Element>),
    EHR(Option<Element>),
    CLUSTER(Option<Element>),
    ACCM(Option<Element>),
    ACCT(Option<Element>),
    ACSN(Option<Element>),
    ADJUD(Option<Element>),
    CACT(Option<Element>),
    CNTRCT(Option<Element>),
    COV(Option<Element>),
    CONC(Option<Element>),
    CONS(Option<Element>),
    CONTREG(Option<Element>),
    CTTEVENT(Option<Element>),
    DISPACT(Option<Element>),
    EXPOS(Option<Element>),
    INC(Option<Element>),
    INFRM(Option<Element>),
    INVE(Option<Element>),
    LIST(Option<Element>),
    MPROT(Option<Element>),
    OBS(Option<Element>),
    ROIBND(Option<Element>),
    ROIOVL(Option<Element>),
    _SubjectBodyPosition(Option<Element>),
    RTRD(Option<Element>),
    TRD(Option<Element>),
    CASE(Option<Element>),
    DETPOL(Option<Element>),
    EXP(Option<Element>),
    LOC(Option<Element>),
    PHN(Option<Element>),
    POL(Option<Element>),
    SEQ(Option<Element>),
    SEQVAR(Option<Element>),
    OBSCOR(Option<Element>),
    POSACC(Option<Element>),
    POSCOORD(Option<Element>),
    PCPR(Option<Element>),
    POLICY(Option<Element>),
    PROC(Option<Element>),
    SPECCOLLECT(Option<Element>),
    REG(Option<Element>),
    REV(Option<Element>),
    SPCTRT(Option<Element>),
    SPLY(Option<Element>),
    STORE(Option<Element>),
    SUBST(Option<Element>),
    TRFR(Option<Element>),
    TRNS(Option<Element>),
    XACT(Option<Element>),
    _ActClassContainer(Option<Element>),
}
pub enum AnimalBreeds {}
pub enum AssertOperatorCodes {}
pub enum RepositoryType {}
pub enum StandardsStatus {}
pub enum V3LanguageAbilityMode {}
pub enum EnteralRoute {}
pub enum UcumBodytemp {}
pub enum MaritalStatus {}
pub enum VisionEyeCodes {}
pub enum ExpressionLanguage {}
pub enum ResourceSecurityCategory {}
pub enum ExDiagnosistype {}
pub enum VerificationresultCommunicationMethod {}
pub enum SpdxLicense {}
pub enum GoalAcceptanceStatus {}
pub enum IssueType {
    Structure(Option<Element>),
    Required(Option<Element>),
    Value(Option<Element>),
    Invariant(Option<Element>),
    Login(Option<Element>),
    Unknown(Option<Element>),
    Expired(Option<Element>),
    Forbidden(Option<Element>),
    Suppressed(Option<Element>),
    NotSupported(Option<Element>),
    Duplicate(Option<Element>),
    MultipleMatches(Option<Element>),
    NotFound(Option<Element>),
    TooLong(Option<Element>),
    CodeInvalid(Option<Element>),
    Extension(Option<Element>),
    TooCostly(Option<Element>),
    BusinessRule(Option<Element>),
    Conflict(Option<Element>),
    LockError(Option<Element>),
    NoStore(Option<Element>),
    Exception(Option<Element>),
    Timeout(Option<Element>),
    Incomplete(Option<Element>),
    Throttled(Option<Element>),
}
pub enum SupplyrequestKind {}
pub enum MedicationknowledgePackageType {}
pub enum ExampleFilter {}
pub enum FlagCategory {}
pub enum DeviceStatusReason {}
pub enum UcumBodylength {}
pub enum V3ParticipationInformationGenerator {
    _ParticipationAncillary(Option<Element>),
    _ParticipationInformationGenerator(Option<Element>),
    ENT(Option<Element>),
    CST(Option<Element>),
    DIR(Option<Element>),
    TPA(Option<Element>),
    NRD(Option<Element>),
    RDV(Option<Element>),
    EXPTRGT(Option<Element>),
    EXSRC(Option<Element>),
    SPC(Option<Element>),
    IND(Option<Element>),
    IRCP(Option<Element>),
    LOC(Option<Element>),
    PRF(Option<Element>),
    RESP(Option<Element>),
    VRF(Option<Element>),
}
pub enum Payeetype {}
pub enum V3EncounterSpecialCourtesy {}
pub enum ClaimException {}
pub enum StructureDefinitionKind {}
pub enum V3ActMood {
    _ActMoodPotential(Option<Element>),
    EVN(Option<Element>),
    INT(Option<Element>),
    _ActMoodActRequest(Option<Element>),
    PRP(Option<Element>),
    APT(Option<Element>),
    CRT(Option<Element>),
    PRMSCRT(Option<Element>),
    RQOCRT(Option<Element>),
    EXPEC(Option<Element>),
    OPT(Option<Element>),
}
pub enum MedicationrequestStatusReason {}
pub enum RelatedArtifactType {}
pub enum V3StyleType {
    Bold(Option<Element>),
    Emphasis(Option<Element>),
    Italics(Option<Element>),
    Underline(Option<Element>),
    _OrderedListStyle(Option<Element>),
    _UnorderedListStyle(Option<Element>),
    Botrule(Option<Element>),
    Lrule(Option<Element>),
    Rrule(Option<Element>),
    Toprule(Option<Element>),
}
pub enum V3QueryStatusCode {}
pub enum CompositionAttestationMode {}
pub enum C80PracticeCodes {}
pub enum V3EntityCode {
    _ContainerEntityType(Option<Element>),
    _NonRigidContainerEntityType(Option<Element>),
    _RigidContainerEntityType(Option<Element>),
    AMP(Option<Element>),
    MINIM(Option<Element>),
    NEBAMP(Option<Element>),
    OVUL(Option<Element>),
    BOT(Option<Element>),
    BOTPLY(Option<Element>),
    BOX(Option<Element>),
    CAN(Option<Element>),
    CART(Option<Element>),
    CNSTR(Option<Element>),
    JAR(Option<Element>),
    JUG(Option<Element>),
    TIN(Option<Element>),
    TUB(Option<Element>),
    TUBE(Option<Element>),
    VIAL(Option<Element>),
    CARD(Option<Element>),
    COMPPKG(Option<Element>),
    KIT(Option<Element>),
    _MedicalDevice(Option<Element>),
    LINE(Option<Element>),
    _InjectionMedicalDevice(Option<Element>),
    APLCTR(Option<Element>),
    INH(Option<Element>),
    PMP(Option<Element>),
    _SpecimenAdditiveEntity(Option<Element>),
    BLDPRD(Option<Element>),
    VCCNE(Option<Element>),
    _DrugEntity(Option<Element>),
    NDA01(Option<Element>),
    NDA02(Option<Element>),
    NDA03(Option<Element>),
    NDA04(Option<Element>),
    NDA05(Option<Element>),
    NDA06(Option<Element>),
    NDA07(Option<Element>),
    NDA08(Option<Element>),
    NDA09(Option<Element>),
    NDA10(Option<Element>),
    NDA11(Option<Element>),
    NDA12(Option<Element>),
    NDA13(Option<Element>),
    NDA14(Option<Element>),
    NDA15(Option<Element>),
    NDA16(Option<Element>),
    NDA17(Option<Element>),
    HHOLD(Option<Element>),
    NAT(Option<Element>),
    RELIG(Option<Element>),
    BED(Option<Element>),
    BLDG(Option<Element>),
    FLOOR(Option<Element>),
    ROOM(Option<Element>),
    WING(Option<Element>),
    PRAC(Option<Element>),
}
pub enum ListOrder {}
pub enum RejectionCriteria {}
pub enum TestscriptProfileDestinationTypes {}
pub enum ExamplescenarioActorType {}
pub enum C80Facilitycodes {}
pub enum BenefitType {}
pub enum V3HL7StandardVersionCode {}
pub enum V3Hl7VoteResolution {
    Affdef(Option<Element>),
    Affi(Option<Element>),
    Affr(Option<Element>),
    Nonsubp(Option<Element>),
    Nonsubv(Option<Element>),
    Notrelp(Option<Element>),
    Notrelv(Option<Element>),
    Prevcons(Option<Element>),
    Retract(Option<Element>),
    Unresolved(Option<Element>),
    Withdraw(Option<Element>),
}
pub enum StudyType {}
pub enum V3ResponseModality {}
pub enum ContactPointSystem {}
pub enum GenderIdentity {}
pub enum MessageheaderResponseRequest {}
pub enum AdverseEventOutcome {}
pub enum EncounterSpecialArrangements {}
pub enum DocumentClasscodes {}
pub enum QuestionnaireUsageMode {}
pub enum AdjudicationError {}
pub enum UsageContextType {}
pub enum FmConditions {}
pub enum ExRevenueCenter {}
pub enum ProcedureFollowup {}
pub enum VerificationresultPushTypeAvailable {}
pub enum CertaintySubcomponentRating {}
pub enum ItemType {
    Boolean(Option<Element>),
    Decimal(Option<Element>),
    Integer(Option<Element>),
    Date(Option<Element>),
    DateTime(Option<Element>),
    Time(Option<Element>),
    String(Option<Element>),
    Text(Option<Element>),
    Url(Option<Element>),
    Choice(Option<Element>),
    OpenChoice(Option<Element>),
    Attachment(Option<Element>),
    Reference(Option<Element>),
    Quantity(Option<Element>),
}
pub enum MedicationrequestCategory {}
pub enum ProvenanceActivityType {
    _ParticipationAncillary(Option<Element>),
    _ParticipationInformationGenerator(Option<Element>),
    ENT(Option<Element>),
    CST(Option<Element>),
    DIR(Option<Element>),
    TPA(Option<Element>),
    NRD(Option<Element>),
    RDV(Option<Element>),
    EXPTRGT(Option<Element>),
    EXSRC(Option<Element>),
    SPC(Option<Element>),
    IND(Option<Element>),
    IRCP(Option<Element>),
    LOC(Option<Element>),
    PRF(Option<Element>),
    RESP(Option<Element>),
    VRF(Option<Element>),
}
pub enum SpecimenCollection {}
pub enum ContractDecisionMode {}
pub enum HistoryAbsentReason {}
pub enum GoalCategory {}
pub enum V3EntityClassManufacturedMaterial {
    HCE(Option<Element>),
    LIV(Option<Element>),
    ANM(Option<Element>),
    MIC(Option<Element>),
    PLNT(Option<Element>),
    MAT(Option<Element>),
    CONT(Option<Element>),
    DEV(Option<Element>),
    ORG(Option<Element>),
    NAT(Option<Element>),
    PLC(Option<Element>),
    RGRP(Option<Element>),
}
pub enum V3ParticipationTargetLocation {
    _ParticipationAncillary(Option<Element>),
    _ParticipationInformationGenerator(Option<Element>),
    ENT(Option<Element>),
    CST(Option<Element>),
    DIR(Option<Element>),
    TPA(Option<Element>),
    NRD(Option<Element>),
    RDV(Option<Element>),
    EXPTRGT(Option<Element>),
    EXSRC(Option<Element>),
    SPC(Option<Element>),
    IND(Option<Element>),
    IRCP(Option<Element>),
    LOC(Option<Element>),
    PRF(Option<Element>),
    RESP(Option<Element>),
    VRF(Option<Element>),
}
pub enum MapInputMode {}
pub enum V3ActRelationshipSplit {}
pub enum V3XBasicConfidentialityKind {}
pub enum V3CommunicationFunctionType {}
pub enum V3EntityClassOrganization {
    HCE(Option<Element>),
    LIV(Option<Element>),
    ANM(Option<Element>),
    MIC(Option<Element>),
    PLNT(Option<Element>),
    MAT(Option<Element>),
    CONT(Option<Element>),
    DEV(Option<Element>),
    ORG(Option<Element>),
    NAT(Option<Element>),
    PLC(Option<Element>),
    RGRP(Option<Element>),
}
pub enum V3ParticipationInformationTranscriber {
    _ParticipationAncillary(Option<Element>),
    _ParticipationInformationGenerator(Option<Element>),
    ENT(Option<Element>),
    CST(Option<Element>),
    DIR(Option<Element>),
    TPA(Option<Element>),
    NRD(Option<Element>),
    RDV(Option<Element>),
    EXPTRGT(Option<Element>),
    EXSRC(Option<Element>),
    SPC(Option<Element>),
    IND(Option<Element>),
    IRCP(Option<Element>),
    LOC(Option<Element>),
    PRF(Option<Element>),
    RESP(Option<Element>),
    VRF(Option<Element>),
}
pub enum V3RoleClassPartitive {
    _RoleClassAssociative(Option<Element>),
    _RoleClassRelationshipFormal(Option<Element>),
    ASSIGNED(Option<Element>),
    CON(Option<Element>),
    GUARD(Option<Element>),
    CLAIM(Option<Element>),
    NAMED(Option<Element>),
    PROG(Option<Element>),
    MIL(Option<Element>),
    CASEBJ(Option<Element>),
    RESBJ(Option<Element>),
    NOT(Option<Element>),
    PROV(Option<Element>),
    CAREGIVER(Option<Element>),
    PRS(Option<Element>),
    SELF(Option<Element>),
    ACCESS(Option<Element>),
    ADJY(Option<Element>),
    BOND(Option<Element>),
    CONY(Option<Element>),
    ADMM(Option<Element>),
    BIRTHPL(Option<Element>),
    DEATHPLC(Option<Element>),
    DST(Option<Element>),
    EXLOC(Option<Element>),
    DSDLOC(Option<Element>),
    ISDLOC(Option<Element>),
    EXPR(Option<Element>),
    HLD(Option<Element>),
    HLTHCHRT(Option<Element>),
    IDENT(Option<Element>),
    MANU(Option<Element>),
    MNT(Option<Element>),
    OWN(Option<Element>),
    RGPR(Option<Element>),
    TERR(Option<Element>),
    USED(Option<Element>),
    WRTE(Option<Element>),
    _RoleClassOntological(Option<Element>),
    SAME(Option<Element>),
    SUBY(Option<Element>),
    GRIC(Option<Element>),
    _RoleClassPartitive(Option<Element>),
    EXPVECTOR(Option<Element>),
    FOMITE(Option<Element>),
    ACTI(Option<Element>),
    ADJV(Option<Element>),
    ADTV(Option<Element>),
    BASE(Option<Element>),
    CNTM(Option<Element>),
    IACT(Option<Element>),
    MECH(Option<Element>),
    STOR(Option<Element>),
    ACTM(Option<Element>),
    ALQT(Option<Element>),
    ISLT(Option<Element>),
}
pub enum V3GTSAbbreviation {
    _GTSAbbreviationHolidaysChristianRoman(Option<Element>),
    JHNNL(Option<Element>),
    JHNUS(Option<Element>),
}
pub enum GoalStatusReason {}
pub enum ObservationStatistics {}
pub enum V3AcknowledgementDetailType {}
pub enum DiagnosticReportStatus {
    Preliminary(Option<Element>),
    Corrected(Option<Element>),
    Appended(Option<Element>),
}
pub enum EncounterParticipantType {
    _ParticipationAncillary(Option<Element>),
    _ParticipationInformationGenerator(Option<Element>),
    ENT(Option<Element>),
    CST(Option<Element>),
    DIR(Option<Element>),
    TPA(Option<Element>),
    NRD(Option<Element>),
    RDV(Option<Element>),
    EXPTRGT(Option<Element>),
    EXSRC(Option<Element>),
    SPC(Option<Element>),
    IND(Option<Element>),
    IRCP(Option<Element>),
    LOC(Option<Element>),
    PRF(Option<Element>),
    RESP(Option<Element>),
    VRF(Option<Element>),
}
pub enum V3ProvenanceEventCurrentStateAS {}
pub enum ListExampleCodes {}
pub enum ServicerequestOrderdetail {}
pub enum AdverseEventCausalityAssess {}
pub enum ImmunizationTargetDisease {}
pub enum BenefitUnit {}
pub enum Iso316612 {}
pub enum DiscriminatorType {}
pub enum InvoiceStatus {}
pub enum V3ActSite {
    _HumanSubstanceAdministrationSite(Option<Element>),
}
pub enum ImmunizationEvaluationStatus {}
pub enum ImmunizationFunction {}
pub enum V3RoleClassMutualRelationship {
    _RoleClassAssociative(Option<Element>),
    _RoleClassRelationshipFormal(Option<Element>),
    ASSIGNED(Option<Element>),
    CON(Option<Element>),
    GUARD(Option<Element>),
    CLAIM(Option<Element>),
    NAMED(Option<Element>),
    PROG(Option<Element>),
    MIL(Option<Element>),
    CASEBJ(Option<Element>),
    RESBJ(Option<Element>),
    NOT(Option<Element>),
    PROV(Option<Element>),
    CAREGIVER(Option<Element>),
    PRS(Option<Element>),
    SELF(Option<Element>),
    ACCESS(Option<Element>),
    ADJY(Option<Element>),
    BOND(Option<Element>),
    CONY(Option<Element>),
    ADMM(Option<Element>),
    BIRTHPL(Option<Element>),
    DEATHPLC(Option<Element>),
    DST(Option<Element>),
    EXLOC(Option<Element>),
    DSDLOC(Option<Element>),
    ISDLOC(Option<Element>),
    EXPR(Option<Element>),
    HLD(Option<Element>),
    HLTHCHRT(Option<Element>),
    IDENT(Option<Element>),
    MANU(Option<Element>),
    MNT(Option<Element>),
    OWN(Option<Element>),
    RGPR(Option<Element>),
    TERR(Option<Element>),
    USED(Option<Element>),
    WRTE(Option<Element>),
    _RoleClassOntological(Option<Element>),
    SAME(Option<Element>),
    SUBY(Option<Element>),
    GRIC(Option<Element>),
    _RoleClassPartitive(Option<Element>),
    EXPVECTOR(Option<Element>),
    FOMITE(Option<Element>),
    ACTI(Option<Element>),
    ADJV(Option<Element>),
    ADTV(Option<Element>),
    BASE(Option<Element>),
    CNTM(Option<Element>),
    IACT(Option<Element>),
    MECH(Option<Element>),
    STOR(Option<Element>),
    ACTM(Option<Element>),
    ALQT(Option<Element>),
    ISLT(Option<Element>),
}
pub enum ImmunizationReason {}
pub enum SpecimenCollectionPriority {}
pub enum SystemRestfulInteraction {}
pub enum V3MessageWaitingPriority {}
pub enum InstanceAvailability {}
pub enum V3IntegrityCheckAlgorithm {}
pub enum AssetAvailability {}
pub enum TimingAbbreviation {}
pub enum PractitionerSpecialty {}
pub enum ProcessPriority {}
pub enum IssueSeverity {}
pub enum ContactPointUse {}
pub enum V3Hl7ITSType {}
pub enum V3ParticipationSignature {}
pub enum AuditSourceType {}
pub enum SubstanceStatus {}
pub enum V3EntityClassRoot {
    HCE(Option<Element>),
    LIV(Option<Element>),
    ANM(Option<Element>),
    MIC(Option<Element>),
    PLNT(Option<Element>),
    MAT(Option<Element>),
    CONT(Option<Element>),
    DEV(Option<Element>),
    ORG(Option<Element>),
    NAT(Option<Element>),
    PLC(Option<Element>),
    RGRP(Option<Element>),
}
pub enum IdentifierType {}
pub enum CarePlanIntent {}
pub enum ExampleExtensional {}
pub enum CompositionStatus {}
pub enum NameV3Representation {}
pub enum Fundsreserve {}
pub enum VerificationresultValidationProcess {}
pub enum V3EntityNamePartTypeR2 {}
pub enum ServiceModifiers {}
pub enum VerificationresultFailureAction {}
pub enum TestscriptProfileOriginTypes {}
pub enum EntformulaType {}
pub enum ContractActionstatus {}
pub enum V3ActRelationshipHasComponent {
    _ActClassTemporallyPertains(Option<Element>),
    _ActRelationshipAccounting(Option<Element>),
    CHRG(Option<Element>),
    COST(Option<Element>),
    CREDIT(Option<Element>),
    DEBIT(Option<Element>),
    _ActRelationshipConditional(Option<Element>),
    BLOCK(Option<Element>),
    DIAG(Option<Element>),
    IMM(Option<Element>),
    MITGT(Option<Element>),
    PRYLX(Option<Element>),
    TREAT(Option<Element>),
    _ActRelationshipTemporallyPertains(Option<Element>),
    ENE(Option<Element>),
    CONCURRENT(Option<Element>),
    SBSECWE(Option<Element>),
    ENS(Option<Element>),
    SNE(Option<Element>),
    SNS(Option<Element>),
    SCWSEBE(Option<Element>),
    SCWSEAE(Option<Element>),
    EAE(Option<Element>),
    SBEEAE(Option<Element>),
    SAS(Option<Element>),
    EAEORECW(Option<Element>),
    OVERLAP(Option<Element>),
    SBSEASEBE(Option<Element>),
    SBE(Option<Element>),
    SBSEBE(Option<Element>),
    EBS(Option<Element>),
    SBS(Option<Element>),
    AUTH(Option<Element>),
    CAUS(Option<Element>),
    COMP(Option<Element>),
    STEP(Option<Element>),
    COVBY(Option<Element>),
    DRIV(Option<Element>),
    ELNK(Option<Element>),
    EVID(Option<Element>),
    EXACBY(Option<Element>),
    EXPL(Option<Element>),
    INTF(Option<Element>),
    ITEMSLOC(Option<Element>),
    LIMIT(Option<Element>),
    META(Option<Element>),
    MFST(Option<Element>),
    NAME(Option<Element>),
    OUTC(Option<Element>),
    OBJC(Option<Element>),
    OBJF(Option<Element>),
    PERT(Option<Element>),
    PREV(Option<Element>),
    REFR(Option<Element>),
    REFV(Option<Element>),
    RELVBY(Option<Element>),
    SEQL(Option<Element>),
    OCCR(Option<Element>),
    OREF(Option<Element>),
    SCH(Option<Element>),
    VRXCRPT(Option<Element>),
    SPRT(Option<Element>),
    SUBJ(Option<Element>),
    SUMM(Option<Element>),
    VALUE(Option<Element>),
}
pub enum ContractAssetsubtype {}
pub enum DefinitionTopic {}
pub enum NamingsystemType {}
pub enum CommonTags {}
pub enum AllergyIntoleranceCriticality {}
pub enum FlagPriority {}
pub enum QuestionnaireItemControl {
    List(Option<Element>),
    Table(Option<Element>),
    Htable(Option<Element>),
    Gtable(Option<Element>),
    Atable(Option<Element>),
    Header(Option<Element>),
    Footer(Option<Element>),
    Inline(Option<Element>),
    Prompt(Option<Element>),
    Unit(Option<Element>),
    Lower(Option<Element>),
    Upper(Option<Element>),
    Flyover(Option<Element>),
    Help(Option<Element>),
    Autocomplete(Option<Element>),
    DropDown(Option<Element>),
    CheckBox(Option<Element>),
    Lookup(Option<Element>),
    RadioButton(Option<Element>),
    Slider(Option<Element>),
    Spinner(Option<Element>),
    TextBox(Option<Element>),
}
pub enum ImmunizationRecommendationDateCriterion {}
pub enum V3ParticipationPhysicalPerformer {
    _ParticipationAncillary(Option<Element>),
    _ParticipationInformationGenerator(Option<Element>),
    ENT(Option<Element>),
    CST(Option<Element>),
    DIR(Option<Element>),
    TPA(Option<Element>),
    NRD(Option<Element>),
    RDV(Option<Element>),
    EXPTRGT(Option<Element>),
    EXSRC(Option<Element>),
    SPC(Option<Element>),
    IND(Option<Element>),
    IRCP(Option<Element>),
    LOC(Option<Element>),
    PRF(Option<Element>),
    RESP(Option<Element>),
    VRF(Option<Element>),
}
pub enum AllTypes {}
pub enum IdentityAssuranceLevel {}
pub enum ExProgramCode {}
pub enum GoalStatus {
    Active(Option<Element>),
    OnHold(Option<Element>),
    Completed(Option<Element>),
}
pub enum ListEmptyReason {}
pub enum CareTeamStatus {}
pub enum QuestionnaireAnswersStatus {}
pub enum SupplyrequestReason {}
pub enum CdshooksIndicator {}
pub enum Forms {}
pub enum QuestionnaireEnableBehavior {}
pub enum V3ProcessingID {}
pub enum ChoiceListOrientation {}
pub enum VerificationresultPrimarySourceType {}
pub enum V3WorkClassificationODH {}
pub enum EligibilityresponsePurpose {}
pub enum V3ActSubstanceAdminSubstitutionCode {
    E(Option<Element>),
    BC(Option<Element>),
    G(Option<Element>),
    TB(Option<Element>),
    TG(Option<Element>),
    F(Option<Element>),
    N(Option<Element>),
}
pub enum V3EmployeeJobClass {}
pub enum DeviceSafety {}
pub enum V3ActMoodPredicate {
    _ActMoodPotential(Option<Element>),
    EVN(Option<Element>),
    INT(Option<Element>),
    _ActMoodActRequest(Option<Element>),
    PRP(Option<Element>),
    APT(Option<Element>),
    CRT(Option<Element>),
    PRMSCRT(Option<Element>),
    RQOCRT(Option<Element>),
    EXPEC(Option<Element>),
    OPT(Option<Element>),
}
pub enum ServicerequestCategory {}
pub enum V3ConfidentialityClassification {}
pub enum V3CalendarType {}
pub enum ResearchStudyPhase {}
pub enum V3TimingEvent {
    CD(Option<Element>),
    CM(Option<Element>),
    CV(Option<Element>),
}
pub enum V3RoleClassPassive {
    _RoleClassAssociative(Option<Element>),
    _RoleClassRelationshipFormal(Option<Element>),
    ASSIGNED(Option<Element>),
    CON(Option<Element>),
    GUARD(Option<Element>),
    CLAIM(Option<Element>),
    NAMED(Option<Element>),
    PROG(Option<Element>),
    MIL(Option<Element>),
    CASEBJ(Option<Element>),
    RESBJ(Option<Element>),
    NOT(Option<Element>),
    PROV(Option<Element>),
    CAREGIVER(Option<Element>),
    PRS(Option<Element>),
    SELF(Option<Element>),
    ACCESS(Option<Element>),
    ADJY(Option<Element>),
    BOND(Option<Element>),
    CONY(Option<Element>),
    ADMM(Option<Element>),
    BIRTHPL(Option<Element>),
    DEATHPLC(Option<Element>),
    DST(Option<Element>),
    EXLOC(Option<Element>),
    DSDLOC(Option<Element>),
    ISDLOC(Option<Element>),
    EXPR(Option<Element>),
    HLD(Option<Element>),
    HLTHCHRT(Option<Element>),
    IDENT(Option<Element>),
    MANU(Option<Element>),
    MNT(Option<Element>),
    OWN(Option<Element>),
    RGPR(Option<Element>),
    TERR(Option<Element>),
    USED(Option<Element>),
    WRTE(Option<Element>),
    _RoleClassOntological(Option<Element>),
    SAME(Option<Element>),
    SUBY(Option<Element>),
    GRIC(Option<Element>),
    _RoleClassPartitive(Option<Element>),
    EXPVECTOR(Option<Element>),
    FOMITE(Option<Element>),
    ACTI(Option<Element>),
    ADJV(Option<Element>),
    ADTV(Option<Element>),
    BASE(Option<Element>),
    CNTM(Option<Element>),
    IACT(Option<Element>),
    MECH(Option<Element>),
    STOR(Option<Element>),
    ACTM(Option<Element>),
    ALQT(Option<Element>),
    ISLT(Option<Element>),
}
pub enum V3EntityNamePartType {}
pub enum ImmunizationFundingSource {}
pub enum VerificationresultValidationStatus {}
pub enum V3ActStatus {
    Aborted(Option<Element>),
    Active(Option<Element>),
    Cancelled(Option<Element>),
    Completed(Option<Element>),
    Held(Option<Element>),
    New(Option<Element>),
    Suspended(Option<Element>),
}
pub enum LdlcholesterolCodes {}
pub enum ReferenceVersionRules {}
pub enum SubscriptionTag {}
pub enum V3MaritalStatus {}
pub enum MedicationAdminStatus {}
pub enum CapabilityStatementKind {}
pub enum Surface {}
pub enum ExposureState {}
pub enum ConceptMapEquivalence {
    Equivalent(Option<Element>),
    Wider(Option<Element>),
    Subsumes(Option<Element>),
    Narrower(Option<Element>),
    Specializes(Option<Element>),
    Inexact(Option<Element>),
    Disjoint(Option<Element>),
}
pub enum ServicePharmacy {}
pub enum V3LocalMarkupIgnore {}
pub enum SpecimenCollectionMethod {}
pub enum ListStatus {}
pub enum PrecisionEstimateType {}
pub enum V3RelationshipConjunction {}
pub enum V3ExposureMode {
    AIRBORNE(Option<Element>),
    CONTACT(Option<Element>),
    FOODBORNE(Option<Element>),
    WATERBORNE(Option<Element>),
}
pub enum ExpansionProcessingRule {}
pub enum V3TargetAwareness {}
pub enum FinancialTaskinputtype {}
pub enum V3ActExposureLevelCode {
    HIGH(Option<Element>),
    LOW(Option<Element>),
    MEDIUM(Option<Element>),
}
pub enum ProcedureOutcome {}
pub enum CatalogType {}
pub enum AddressUse {}
pub enum DaysOfWeek {}
pub enum ResourceSlicingRules {}
pub enum GoalPriority {}
pub enum V3EntityClassLivingSubject {
    HCE(Option<Element>),
    LIV(Option<Element>),
    ANM(Option<Element>),
    MIC(Option<Element>),
    PLNT(Option<Element>),
    MAT(Option<Element>),
    CONT(Option<Element>),
    DEV(Option<Element>),
    ORG(Option<Element>),
    NAT(Option<Element>),
    PLC(Option<Element>),
    RGRP(Option<Element>),
}
pub enum DefinitionUse {}
pub enum ConceptPropertyType {}
pub enum InvoicePriceComponentType {}
pub enum DoseRateType {}
pub enum DocumentReferenceStatus {}
pub enum HistoryStatus {}
pub enum MeasurePopulation {}
pub enum LibraryType {}
pub enum BodystructureRelativeLocation {}
pub enum V3NullFlavor {
    INV(Option<Element>),
    NINF(Option<Element>),
    PINF(Option<Element>),
    MSK(Option<Element>),
    NA(Option<Element>),
    UNK(Option<Element>),
    NAV(Option<Element>),
}
pub enum EndpointConnectionType {}
pub enum SupplementType {}
pub enum V3TransmissionRelationshipTypeCode {}
pub enum EventStatus {}
pub enum MapContextType {}
pub enum PerformerFunction {}
pub enum ActionPrecheckBehavior {}
pub enum Languages {}
pub enum MetricCalibrationState {}
pub enum ResourceTypes {}
pub enum V3EntityHandling {}
pub enum MediaType {}
pub enum V3Hl7PublishingSubSection {}
pub enum AdverseEventCategory {
    WrongDose(Option<Element>),
    IncorrectPrescribingInformation(Option<Element>),
    WrongTechnique(Option<Element>),
    WrongRouteOfAdministration(Option<Element>),
    WrongRate(Option<Element>),
    WrongDuration(Option<Element>),
    WrongTime(Option<Element>),
    ExpiredDrug(Option<Element>),
}
pub enum MessageSignificanceCategory {}
pub enum ContractSecurityControl {}
pub enum MedicationrequestIntent {}
pub enum V3ActClassInvestigation {
    _ActClassRecordOrganizer(Option<Element>),
    DOC(Option<Element>),
    CDALVLONE(Option<Element>),
    CATEGORY(Option<Element>),
    DOCBODY(Option<Element>),
    DOCSECT(Option<Element>),
    TOPIC(Option<Element>),
    EHR(Option<Element>),
    CLUSTER(Option<Element>),
    ACCM(Option<Element>),
    ACCT(Option<Element>),
    ACSN(Option<Element>),
    ADJUD(Option<Element>),
    CACT(Option<Element>),
    CNTRCT(Option<Element>),
    COV(Option<Element>),
    CONC(Option<Element>),
    CONS(Option<Element>),
    CONTREG(Option<Element>),
    CTTEVENT(Option<Element>),
    DISPACT(Option<Element>),
    EXPOS(Option<Element>),
    INC(Option<Element>),
    INFRM(Option<Element>),
    INVE(Option<Element>),
    LIST(Option<Element>),
    MPROT(Option<Element>),
    OBS(Option<Element>),
    ROIBND(Option<Element>),
    ROIOVL(Option<Element>),
    _SubjectBodyPosition(Option<Element>),
    RTRD(Option<Element>),
    TRD(Option<Element>),
    CASE(Option<Element>),
    DETPOL(Option<Element>),
    EXP(Option<Element>),
    LOC(Option<Element>),
    PHN(Option<Element>),
    POL(Option<Element>),
    SEQ(Option<Element>),
    SEQVAR(Option<Element>),
    OBSCOR(Option<Element>),
    POSACC(Option<Element>),
    POSCOORD(Option<Element>),
    PCPR(Option<Element>),
    POLICY(Option<Element>),
    PROC(Option<Element>),
    SPECCOLLECT(Option<Element>),
    REG(Option<Element>),
    REV(Option<Element>),
    SPCTRT(Option<Element>),
    SPLY(Option<Element>),
    STORE(Option<Element>),
    SUBST(Option<Element>),
    TRFR(Option<Element>),
    TRNS(Option<Element>),
    XACT(Option<Element>),
    _ActClassContainer(Option<Element>),
}
pub enum V3EntityRisk {
    EXP(Option<Element>),
    BHZ(Option<Element>),
}
pub enum ContractDefinitionSubtype {}
pub enum CommunicationTopic {}
pub enum KnowledgeResourceTypes {}
pub enum DeviceNametype {}
