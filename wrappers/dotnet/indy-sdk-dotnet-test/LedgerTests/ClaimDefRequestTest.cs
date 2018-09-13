﻿using Hyperledger.Indy.LedgerApi;
using Hyperledger.Indy.DidApi;
using Microsoft.VisualStudio.TestTools.UnitTesting;
using Newtonsoft.Json.Linq;
using System.Threading.Tasks;

namespace Hyperledger.Indy.Test.LedgerTests
{
    [TestClass]
    public class ClaimDefRequestTest : IndyIntegrationTestWithPoolAndSingleWallet
    {
        private const string _claimDefTemplate = "{{\n" +
        "            \"ref\":{0},\n" +
        "            \"signature_type\":\"CL\",\n" +
        "            \"origin\":\"{1}\",\n" +
        "            \"data\":{{\n" +
        "                \"primary\":{{\n" +
        "                    \"n\":\"83469852984476956871633111285697420678256060723156580163068122759469567425381600849138438902552107548539766861666590365174848381535291010418041757276710240953030842046122202402016906205924972182252295487319094577329593677544393592632224714613427822130473474379696616183721440743475053734247824037725487533789856061706740833324717788602268746116297029721621398888459529131593826880823126900285858832457134377949183677639585442886904844793608783831753240185678448312284269486845497720949217396146132958861735347072722092449280372574205841746312833280031873247525372459800132930201998084029506922484661426185450002143461\",\n" +
        "                    \"s\":\"36598527821865478336201608644402887021319976830281254144922838415193047189326184120876650311572665920640111967758238425066565864958446810892401358531545590342401290056836149703549220109981509774843525259400920352082531560361277411808530872594109525982462491998670199872903823657869742599086495624835178373073050767142081484206776345277546531080450529061958937980460303537107061046725579009809137197541389237618812289642185603461102513124991949835948586623327143696280240600789928383168220919049794681181861776889681393339729944540218566460627715413465709316412838042632482652979005394086058441511591756153781159121227\",\n" +
        "                    \"rms\":\"23836382972046033209463023456985914927629254782286444334728987813724281765327660893337383450653748691133061782449580026414785334582859397732571499366000805280757877601804441568263743400086744823885203441870748890135445454347495577599234477813361254101857848089907496868136222777024967328411073984887059985103475100012787049491016895625234124538894645853939428009610771524580099452739392988318945585946758355611531582519514003714424216836334706370901576611410508113637778751976890941210538254418937285168453791223070083264852447713991114719905171445881819334587600693321106919667204512182250084258986170095774914769107\",\n" +
        "                    \"r\":{{\n" +
        "                        \"age\":\"15428480888651268593621235736458685943389726269437020388313417035842991073151072061010468945249435098482625869236498750525662874597991333642865524104221652457788998109101749530884821300954337535472137069380551054204373136155978715752232238326100335828797868667735730830741789880726890058203015780792583471770404478023662994191588489722949795849990796063953164194432710764145637578113087142419634074378464118254848566088943085760634805903735300398689750649630456000759025366784986694635635510206166144055869823907120081668956271923743188342071093889666111639924270726451727101864752767708690529389259470017692442002767\",\n" +
        "                        \"name\":\"74008461204977705404956807338714891429397387365673402608947856456696416827848931951447004905350314563364442667940680669672331872875260077257474781261367591510351742401708951175978700805098470304211391452678992053755170054677498844656517106987419550598382601263743442309896367374279461481792679346472671426558385003925845431219156475505665973289508268634194964491418902859845301351562575713510002838692825728016254605821829245646855474149449192539144107522081712005891593405826343897070114168186645885993480245755494685105636168333649181939830898090651120926156152753918086493335382129839850609934233944397884745134858\",\n" +
        "                        \"sex\":\"40646934914193532541511585946883243600955533193734077080129022860038019728021796610599139377923881754708640252789475144625086755150150612623804964347576907276705600241268266301487516824189453869193926251791711672689649199860304727280764676403810510047397326018392955950249975529169980045664874433828266623495515931483137318724210483205730962036282580749206735450480976847188040030165278917936054139926609849181885654646269083459580415131952938813839182742590617440550773580790446467896469090164142755499827316294406540664375065617280568303837662431668218593808092907551353093044984225946834165309588512359379032847125\",\n" +
        "                        \"height\":\"60077310006190228182950362501472785016827894350517184186566050586806482282196022414888288252599211919680339352529750982779980002923071031258837648242708410943080288964834346858544959217874890558006056551949543916094446891954292824146212277550956558692224016968153138097595802008943263818064605343108607131298420107448075252583276684858815371561492996587478784667827675142382692061950832554910690663724101360454494298013020706080246986445114235542283015624510836206522238238728405826712730615187235709554561445384409566940622412591208469650855059870671771721035756205878334354791247751663679130847366069215369484993653\"\n" +
        "                    }},\n" +
        "                    \"rctxt\":\"36378575722516953828830668112614685244571602424420162720244033008706985740860180373728219883172046821464173434592331868657297711725743060654773725561634332269874655603697872022630999786617840856366807034806938874090561725454026277048301648000835861491030368037108847175790943895107305383779598585532854170748970999977490244057635358075906501562932970296830906796719844887269636297064678777638050708353254894155336111384638276041851818109156194135995350001255928374102089400812445206030019103440866343736761861049159446083221399575945128480681798837648578166327164995640582517916904912672875184928940552983440410245037\",\n" +
        "                    \"z\":\"65210811645114955910002482704691499297899796787292244564644467629838455625296674951468505972574512639263601600908664306008863647466643899294681985964775001929521624341158696866597713112430928402519124073453804861185882073381514901830347278653016300430179820703804228663001232136885036042101307423527913402600370741689559698469878576457899715687929448757963179899698951620405467976414716277505767979494596626867505828267832223147104774684678295400387894506425769550011471322118172087199519094477785389750318762521728398390891214426117908390767403438354009767291938975195751081032073309083309746656253788033721103313036\"\n" +
        "                }}\n" +
        "            }}\n" +
        "        }}";

        private const string _signatureType = "CL";
        private const int _seqNo = 1;

        [TestMethod]
        public async Task TestBuildClaimDefRequestWorks()
        {
            var dataTemplate = "{{\"primary\":{{\"n\":\"1\",\"s\":\"2\",\"rms\":\"3\",\"r\":{{\"name\":\"1\"}},\"rctxt\":\"1\",\"z\":\"1\"}}{0}}}";
            var data = string.Format(dataTemplate, string.Empty);

            var expectedData = string.Format(dataTemplate, ",\"revocation\":{}");

            var expectedResult = string.Format("\"identifier\":\"{0}\"," +
                    "\"operation\":{{" +
                    "\"ref\":{1}," +
                    "\"data\":{2}," +
                    "\"type\":\"102\"," +
                    "\"signature_type\":\"{3}\"" +
                    "}}", DID1, _seqNo, expectedData, _signatureType);

            var claimDefRequest = await Ledger.BuildClaimDefTxnAsync(DID1, _seqNo, _signatureType, data);


            Assert.IsTrue(claimDefRequest.Replace("\\", "").Contains(expectedResult));
        }

        [TestMethod]
        public async Task TestBuildGetClaimDefRequestWorks()
        {
            var expectedResult = string.Format("\"identifier\":\"{0}\"," +
                    "\"operation\":{{" +
                    "\"type\":\"108\"," +
                    "\"ref\":{1}," +
                    "\"signature_type\":\"{2}\"," +
                    "\"origin\":\"{3}\"" +
                    "}}", DID1, _seqNo, _signatureType, DID1);

            var getClaimDefRequest = await Ledger.BuildGetClaimDefTxnAsync(DID1, _seqNo, _signatureType, DID1);


            Assert.IsTrue(getClaimDefRequest.Replace("\\", "").Contains(expectedResult));
        }

        [TestMethod]
        public async Task TestBuildClaimDefRequestWorksForInvalidJson()
        {
            var data = "{\"primary\":{\"n\":\"1\",\"s\":\"2\",\"rms\":\"3\",\"r\":{\"name\":\"1\"}}}";

            var ex = await Assert.ThrowsExceptionAsync<InvalidStructureException>(() =>
                Ledger.BuildClaimDefTxnAsync(DID1, _seqNo, _signatureType, data)
            );            
        }

        [TestMethod] 
        public async Task TestClaimDefRequestWorks()
        {
            var trusteeDidResult = await Did.CreateAndStoreMyDidAsync(wallet, TRUSTEE_IDENTITY_JSON);
            var trusteeDid = trusteeDidResult.Did;

            var myDidResult = await Did.CreateAndStoreMyDidAsync(wallet, "{}");
            var myDid = myDidResult.Did;
            var myVerkey = myDidResult.VerKey;

            var nymRequest = await Ledger.BuildNymRequestAsync(trusteeDid, myDid, myVerkey, null, null);
            await Ledger.SignAndSubmitRequestAsync(pool, wallet, trusteeDid, nymRequest);

            var claimDef = string.Format(_claimDefTemplate, _seqNo, myDid);

            var claimDefObj = JObject.Parse(claimDef);

            var claimDefJson = claimDefObj["data"].ToString();
            var claimDefRequest = ""; //TODO await Ledger.BuildClaimDefTxnAsync(myDid, _seqNo, _signatureType, claimDefJson);
            await Ledger.SignAndSubmitRequestAsync(pool, wallet, myDid, claimDefRequest);
   
            var getClaimDefRequest = ""; //TODO await Ledger.BuildGetClaimDefTxnAsync(myDid, _seqNo, _signatureType, claimDefObj["origin"].ToString());
            var getClaimDefResponse = await Ledger.SubmitRequestAsync(pool, getClaimDefRequest);

            var getClaimDefResponseObj = JObject.Parse(getClaimDefResponse);

            var expectedClaimDef = claimDefObj["data"]["primary"];
            var actualClaimDef = getClaimDefResponseObj["result"]["data"]["primary"];

            Assert.AreEqual(expectedClaimDef["n"], actualClaimDef["n"]);
            Assert.AreEqual(expectedClaimDef["rms"], actualClaimDef["rms"]);
            Assert.AreEqual(expectedClaimDef["rctxt"], actualClaimDef["rctxt"]);
            Assert.AreEqual(expectedClaimDef["z"], actualClaimDef["z"]);
            Assert.AreEqual(expectedClaimDef["n"], actualClaimDef["n"]);

            var expectedR = (JObject)expectedClaimDef["r"];
            var actualR = (JObject)actualClaimDef["r"];

            Assert.AreEqual(expectedR.Value<string>("age"), actualR.Value<string>("age"));
            Assert.AreEqual(expectedR.Value<string>("sex"), actualR.Value<string>("sex"));
            Assert.AreEqual(expectedR.Value<string>("height"), actualR.Value<string>("height"));
            Assert.AreEqual(expectedR.Value<string>("name"), actualR.Value<string>("name"));
        }

        [TestMethod]
        public async Task TestClaimDefRequestWorksWithoutSignature()
        {
            var trusteeDidResult = await Did.CreateAndStoreMyDidAsync(wallet, TRUSTEE_IDENTITY_JSON);
            var trusteeDid = trusteeDidResult.Did;

            var myDidResult = await Did.CreateAndStoreMyDidAsync(wallet, "{}");
            var myDid = myDidResult.Did;
            var myVerkey = myDidResult.VerKey;

            var nymRequest = await Ledger.BuildNymRequestAsync(trusteeDid, myDid, myVerkey, null, null);
            await Ledger.SignAndSubmitRequestAsync(pool, wallet, trusteeDid, nymRequest);
            
            var claimDef = string.Format(_claimDefTemplate, _seqNo, myDid);

            var claimDefObj = JObject.Parse(claimDef);
            var claimDefJson = claimDefObj["data"].ToString();

            var claimDefRequest = ""; //TODO await Ledger.BuildClaimDefTxnAsync(myDid, _seqNo, _signatureType, claimDefJson);

            var ex = await Assert.ThrowsExceptionAsync<InvalidLedgerTransactionException>(() =>
                Ledger.SubmitRequestAsync(pool, claimDefRequest)
            );
        }
    }
}
